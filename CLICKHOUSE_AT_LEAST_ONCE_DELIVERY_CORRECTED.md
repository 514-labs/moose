# ClickHouse At Least Once Delivery Implementation (Corrected)

## Overview

Based on the ClickHouse support response, this document describes the **corrected** implementation of at least once delivery guarantees for INSERT commands and proper acknowledgment of DDL schema changes in the Moose Framework CLI.

## ClickHouse Support Response Summary

According to ClickHouse support:

> Because of limitations of the HTTP protocol, a HTTP 200 response code does not guarantee that a query was successful.

**Solution**: Add `wait_end_of_query=1` parameter to HTTP requests to ensure:
1. At least once delivery for INSERT commands
2. Proper acknowledgment of DDL changes before proceeding to the next operation

## ⚠️ Critical Design Decision: Selective Application

**IMPORTANT**: `wait_end_of_query=1` is **ONLY** applied to INSERT and DDL operations, **NOT** to SELECT queries.

### Why This Selective Approach is Essential

`wait_end_of_query=1` forces response buffering, which would **severely impact** SELECT queries by:

- **Breaking streaming results**: Client waits for entire response before returning anything
- **Increasing memory usage**: Full result set must be buffered in memory/disk
- **Increasing latency**: No results visible until query completely finishes
- **Reducing concurrency**: Multiple SELECT queries block each other
- **Degrading user experience**: Real-time dashboards and interactive queries become unresponsive

### Documentation Evidence

From ClickHouse HTTP Interface Documentation:

> "To ensure that the entire response is buffered, set `wait_end_of_query=1`. In this case, the data that is not stored in memory will be buffered in a temporary server file."

This buffering behavior is **beneficial** for INSERT/DDL operations (ensures durability) but **harmful** for SELECT operations (breaks streaming and performance).

## Corrected Implementation

### 1. Rust ClickHouse Client (`apps/framework-cli/src/infrastructure/olap/clickhouse/client.rs`)

**Previous (Problematic) Implementation:**
```rust
// Applied to ALL operations - WRONG
let params = &[
    ("query", query),
    ("date_time_input_format", "best_effort"),
    ("wait_end_of_query", "1"), // This breaks SELECT performance!
];
```

**Corrected Implementation:**
```rust
fn query_param(query: &str) -> anyhow::Result<String> {
    let mut params = vec![
        ("query", query),
        ("date_time_input_format", "best_effort"),
    ];
    
    // Only add wait_end_of_query for INSERT and DDL operations
    let query_upper = query.trim().to_uppercase();
    if query_upper.starts_with("INSERT") || 
       query_upper.starts_with("CREATE") || 
       query_upper.starts_with("ALTER") || 
       query_upper.starts_with("DROP") {
        params.push(("wait_end_of_query", "1"));
    }
    
    let encoded = serde_urlencoded::to_string(&params)?;
    Ok(encoded)
}
```

**Impact:**
- ✅ `insert()` method: At least once delivery guaranteed
- ✅ `execute_sql()` with DDL: Proper acknowledgment of schema changes
- ✅ `execute_sql()` with SELECT: Preserves streaming performance

### 2. TypeScript ClickHouse Client (`packages/ts-moose-lib/src/commons.ts`)

**Previous (Problematic) Implementation:**
```typescript
// Global setting affects ALL operations - WRONG
return createClient({
  // ...
  clickhouse_settings: {
    wait_end_of_query: 1, // This breaks ALL SELECT queries!
  },
});
```

**Corrected Implementation:**
```typescript
export const getClickhouseClient = ({...}: ClientConfig) => {
  return createClient({
    url: `${protocol}://${host}:${port}`,
    username: username,
    password: password,
    database: database,
    application: "moose",
    // Note: wait_end_of_query is configured per operation type, not globally
    // to preserve SELECT query performance while ensuring INSERT/DDL reliability
  });
};
```

**Impact:** Each operation type now configures the parameter individually as needed.

### 3. TypeScript OlapTable (`packages/ts-moose-lib/src/dmv2/sdk/olapTable.ts`)

**Implementation:** ✅ **Correctly targets INSERT operations only**
```typescript
private prepareInsertOptions(...): any {
  const insertOptions: any = {
    table: tableName,
    format: "JSONEachRow",
    clickhouse_settings: {
      date_time_input_format: "best_effort",
      wait_end_of_query: 1, // ✅ Only for INSERT operations
      // ...
    },
  };
  // ...
}
```

### 4. TypeScript Consumption APIs (`packages/ts-moose-lib/src/consumption-apis/helpers.ts`)

**Previous (Problematic) Implementation:**
```typescript
// This is used for SELECT queries - WRONG
return this.client.query({
  query,
  query_params,
  format: "JSONEachRow",
  query_id: this.query_id_prefix + randomUUID(),
  clickhouse_settings: {
    wait_end_of_query: 1, // This breaks SELECT streaming!
  },
});
```

**Corrected Implementation:**
```typescript
async execute<T = any>(sql: Sql): Promise<ResultSet<"JSONEachRow"> & { __query_result_t?: T[] }> {
  const [query, query_params] = toQuery(sql);

  return this.client.query({
    query,
    query_params,
    format: "JSONEachRow",
    query_id: this.query_id_prefix + randomUUID(),
    // Note: wait_end_of_query deliberately NOT set here as this is used for SELECT queries
    // where response buffering would harm streaming performance and concurrency
  });
}
```

## Summary of Benefits

### ✅ What We Achieved
1. **At least once delivery for INSERT operations** - Data integrity guaranteed
2. **Proper DDL acknowledgment** - Schema changes confirmed before proceeding
3. **Preserved SELECT query performance** - Streaming results, low latency, high concurrency
4. **Targeted approach** - Each operation type optimized for its specific needs

### ❌ What We Avoided
1. **SELECT query performance degradation** - No forced buffering
2. **Memory bloat** - Large result sets don't require full buffering
3. **Latency spikes** - Results stream as they're produced
4. **Concurrency bottlenecks** - Multiple SELECT queries don't block each other

## Testing Recommendations

### Test INSERT Operations
```bash
# Should include wait_end_of_query=1
curl -v "http://localhost:8123/" -d "INSERT INTO test_table VALUES (1, 'test')"
```

### Test SELECT Operations  
```bash
# Should NOT include wait_end_of_query (preserves streaming)
curl -v "http://localhost:8123/" -d "SELECT * FROM test_table"
```

### Test DDL Operations
```bash
# Should include wait_end_of_query=1
curl -v "http://localhost:8123/" -d "CREATE TABLE test_table2 (id Int32, name String) ENGINE = MergeTree() ORDER BY id"
```

## Conclusion

The corrected implementation ensures **at least once delivery** for data modification operations while preserving the **performance characteristics** that make ClickHouse excel at analytical workloads. This selective approach is essential for maintaining system responsiveness while guaranteeing data integrity where it matters most.

**Key Principle**: Apply reliability guarantees where needed (INSERT/DDL) without compromising performance where it's critical (SELECT).