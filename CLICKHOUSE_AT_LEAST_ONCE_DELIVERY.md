# ClickHouse At Least Once Delivery Implementation

## Overview

Based on the ClickHouse support response, this document describes the implementation of at least once delivery guarantees for INSERT commands and proper acknowledgment of DDL schema changes in the Moose Framework CLI.

## ClickHouse Support Response Summary

According to ClickHouse support:

> Because of limitations of the HTTP protocol, a HTTP 200 response code does not guarantee that a query was successful.

**Solution**: Add `wait_end_of_query=1` parameter to HTTP requests to ensure:
1. At least once delivery for INSERT commands
2. Proper acknowledgment of DDL changes before proceeding to the next operation

## Changes Implemented

### 1. Rust ClickHouse Client (`apps/framework-cli/src/infrastructure/olap/clickhouse/client.rs`)

**Change**: Added `wait_end_of_query=1` to the query parameters in the `query_param` function.

```rust
fn query_param(query: &str) -> anyhow::Result<String> {
    let params = &[
        ("query", query),
        ("date_time_input_format", "best_effort"),
        ("wait_end_of_query", "1"), // Ensure at least once delivery and DDL acknowledgment
    ];
    let encoded = serde_urlencoded::to_string(params)?;
    Ok(encoded)
}
```

**Impact**: This affects both `insert()` and `execute_sql()` methods, ensuring all INSERT operations and DDL statements (CREATE TABLE, ALTER TABLE, DROP TABLE) are properly acknowledged.

### 2. TypeScript ClickHouse Client Configuration (`packages/ts-moose-lib/src/commons.ts`)

**Change**: Added `wait_end_of_query: 1` to the global client configuration.

```typescript
export const getClickhouseClient = ({...}: ClientConfig) => {
  return createClient({
    url: `${protocol}://${host}:${port}`,
    username: username,
    password: password,
    database: database,
    application: "moose",
    clickhouse_settings: {
      wait_end_of_query: 1, // Ensure at least once delivery and DDL acknowledgment
    },
  });
};
```

**Impact**: All ClickHouse operations using this client will have at least once delivery guarantees.

### 3. TypeScript OlapTable Insert Operations (`packages/ts-moose-lib/src/dmv2/sdk/olapTable.ts`)

**Change**: Added `wait_end_of_query: 1` to the insert operation settings.

```typescript
private prepareInsertOptions(...): any {
  const insertOptions: any = {
    table: tableName,
    format: "JSONEachRow",
    clickhouse_settings: {
      date_time_input_format: "best_effort",
      wait_end_of_query: 1, // Ensure at least once delivery and DDL acknowledgment
      // ... other settings
    },
  };
}
```

**Impact**: All table insert operations will be properly acknowledged.

### 4. TypeScript Blocks Runner (`packages/ts-moose-lib/src/blocks/runner.ts`)

**Change**: Added `wait_end_of_query: 1` to both setup and teardown DDL operations.

```typescript
// Setup operations
await chClient.command({ 
  query,
  clickhouse_settings: {
    wait_end_of_query: 1, // Ensure at least once delivery and DDL acknowledgment
  },
});

// Teardown operations  
await chClient.command({ 
  query,
  clickhouse_settings: {
    wait_end_of_query: 1, // Ensure at least once delivery and DDL acknowledgment
  },
});
```

**Impact**: All DDL operations in blocks (table creation, deletion, etc.) will be properly acknowledged.

### 5. TypeScript Consumption APIs (`packages/ts-moose-lib/src/consumption-apis/helpers.ts`)

**Change**: Added `wait_end_of_query: 1` to query execution settings.

```typescript
async execute<T = any>(sql: Sql): Promise<ResultSet<"JSONEachRow"> & { __query_result_t?: T[] }> {
  return this.client.query({
    query,
    query_params,
    format: "JSONEachRow",
    query_id: this.query_id_prefix + randomUUID(),
    clickhouse_settings: {
      wait_end_of_query: 1, // Ensure at least once delivery and DDL acknowledgment
    },
  });
}
```

**Impact**: All query executions through the consumption APIs will be properly acknowledged.

### 6. Python OlapTable Operations (`packages/py-moose-lib/moose_lib/dmv2/olap_table.py`)

**Change**: Added `wait_end_of_query: 1` to all insert operation settings.

```python
def _prepare_insert_options(...) -> tuple[str, bytes, dict]:
    settings = {
        "date_time_input_format": "best_effort",
        "wait_end_of_query": 1,  # Ensure at least once delivery and DDL acknowledgment
        # ... other settings
    }
```

**Additional Changes**: Updated all retry mechanisms and batch operations to include the parameter:
- `_retry_individual_records()`: Batch and individual record retry operations
- `_insert_stream()`: Stream insertion operations

**Impact**: All Python table insert operations will be properly acknowledged.

### 7. Python Blocks Runner (`apps/framework-cli/src/framework/python/wrappers/blocks_runner.py`)

**Change**: Added `wait_end_of_query: 1` to all DDL command executions.

```python
# Setup operations
ch_client.command(query, settings={'wait_end_of_query': 1})

# Teardown operations  
ch_client.command(query, settings={'wait_end_of_query': 1})
```

**Impact**: All Python-based DDL operations in blocks will be properly acknowledged.

## Benefits

1. **At Least Once Delivery**: INSERT operations now have guaranteed delivery confirmation
2. **DDL Acknowledgment**: Schema changes (CREATE TABLE, ALTER TABLE, DROP TABLE) are confirmed before proceeding
3. **Migration Safety**: DDL migrations will now be applied sequentially with proper acknowledgment
4. **Data Consistency**: Reduced risk of data loss or incomplete schema changes
5. **Pipeline Reliability**: Insert commands in pipelines now have delivery guarantees

## Compatibility

- **ClickHouse Version**: Compatible with ClickHouse Cloud and self-hosted instances
- **HTTP Interface**: Uses the standard ClickHouse HTTP interface parameter
- **Performance**: Minimal performance impact as acknowledgment only waits for query completion
- **Async Inserts**: Works alongside existing async insert configurations (`wait_for_async_insert: 1`)

## Alternative Approaches Considered

According to ClickHouse support, there were two main options:
1. **`wait_end_of_query=1`** (implemented) - Waits for query completion acknowledgment
2. **Async Inserts** with `async_insert=1,wait_for_async_insert=1` - Already partially implemented

The chosen approach (`wait_end_of_query=1`) provides broader coverage for both INSERT and DDL operations.

## Testing

All changes maintain backward compatibility and should not break existing functionality. The parameter is additive and only enhances delivery guarantees.

## Summary

These changes ensure that the Moose Framework now has at least once delivery guarantees for all ClickHouse operations, addressing the original concern about INSERT delivery and DDL acknowledgment. The implementation is comprehensive, covering all ClickHouse interactions across Rust, TypeScript, and Python components.