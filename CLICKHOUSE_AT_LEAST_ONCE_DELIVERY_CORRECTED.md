# ClickHouse At Least Once Delivery Implementation (Corrected)

## Overview

Based on the ClickHouse support response, this document describes the **corrected** implementation of at least once delivery guarantees for INSERT commands and proper acknowledgment of DDL schema changes in the Moose Framework CLI.

## ClickHouse Support Response Summary

According to ClickHouse support:

> Because of limitations of the HTTP protocol, a HTTP 200 response code does not guarantee that a query was successful.

**Solution**: Add `wait_end_of_query=1` parameter to HTTP requests to ensure:
1. At least once delivery for INSERT commands
2. Proper acknowledgment of DDL changes before proceeding to the next operation

## ⚠️ Critical Design Principle

**`wait_end_of_query=1` MUST ONLY be applied to INSERT and DDL operations, NOT SELECT queries**, because:

- **Response Buffering**: Forces ClickHouse to buffer the entire response before returning
- **Breaks Streaming**: SELECT queries lose their streaming capability
- **Increases Latency**: Users see no results until the entire query completes
- **Reduces Concurrency**: Multiple SELECT queries will block each other
- **Increases Memory Usage**: Full result sets must be buffered

## ✅ Corrected Implementation

### 1. Rust ClickHouse Client (`apps/framework-cli/src/infrastructure/olap/clickhouse/client.rs`)

**Changes Made:**
- ✅ Added constant for DDL commands: `const DDL_COMMANDS: &[&str] = &["INSERT", "CREATE", "ALTER", "DROP", "TRUNCATE"]`
- ✅ Modified `query_param()` to conditionally add `wait_end_of_query=1` only for INSERT/DDL operations
- ✅ Added comprehensive unit tests to verify correct behavior
- ✅ Preserves SELECT query performance by avoiding response buffering

```rust
const DDL_COMMANDS: &[&str] = &["INSERT", "CREATE", "ALTER", "DROP", "TRUNCATE"];

fn query_param(query: &str) -> anyhow::Result<String> {
    let mut params = vec![
        ("query", query),
        ("date_time_input_format", "best_effort"),
    ];
    
    // Only add wait_end_of_query for INSERT and DDL operations
    let query_upper = query.trim().to_uppercase();
    if DDL_COMMANDS.iter().any(|cmd| query_upper.starts_with(cmd)) {
        params.push(("wait_end_of_query", "1"));
    }
    
    let encoded = serde_urlencoded::to_string(&params)?;
    Ok(encoded)
}
```

### 2. TypeScript ClickHouse Client (`packages/ts-moose-lib/src/commons.ts`)

**Changes Made:**
- ✅ Removed global `wait_end_of_query` setting from client configuration
- ✅ Added documentation explaining per-operation configuration approach

```typescript
export const getClickhouseClient = ({ ... }: ClientConfig) => {
  return createClient({
    // ... client config
    // Note: wait_end_of_query is configured per-operation, not globally
    // to preserve SELECT query performance while ensuring INSERT/DDL reliability
  });
};
```

### 3. TypeScript OlapTable (`packages/ts-moose-lib/src/dmv2/sdk/olapTable.ts`)

**Changes Made:**
- ✅ Added `wait_end_of_query: 1` specifically to INSERT operations
- ✅ Updated comment to accurately reflect INSERT-only context

```typescript
clickhouse_settings: {
  wait_end_of_query: 1, // Ensure at least once delivery for INSERT operations
  // ... other INSERT-specific settings
}
```

### 4. TypeScript Consumption APIs (`packages/ts-moose-lib/src/consumption-apis/helpers.ts`)

**Changes Made:**
- ✅ **Removed** `wait_end_of_query` from SELECT query execution
- ✅ Added explanatory comment about why it's deliberately omitted

```typescript
async execute<T = any>(sql: Sql): Promise<ResultSet<"JSONEachRow">> {
  return this.client.query({
    query, query_params, format: "JSONEachRow",
    // Note: wait_end_of_query deliberately NOT set here as this is used for SELECT queries
    // where response buffering would harm streaming performance and concurrency
  });
}
```

### 5. TypeScript Blocks Runner (`packages/ts-moose-lib/src/blocks/runner.ts`)

**Changes Made:**
- ✅ Added `wait_end_of_query: 1` to DDL operations (CREATE/DROP blocks)
- ✅ Maintains accurate DDL acknowledgment comments

```typescript
await chClient.command({ 
  query,
  clickhouse_settings: {
    wait_end_of_query: 1, // Ensure at least once delivery and DDL acknowledgment
  },
});
```

### 6. Python OlapTable (`packages/py-moose-lib/moose_lib/dmv2/olap_table.py`)

**Changes Made:**
- ✅ Added `wait_end_of_query: 1` to all INSERT operations
- ✅ Updated comments to reflect INSERT-only context
- ✅ Applied to both regular inserts and retry logic
- ✅ Added comprehensive unit tests

```python
settings = {
    "date_time_input_format": "best_effort",
    "wait_end_of_query": 1,  # Ensure at least once delivery for INSERT operations
    # ... other INSERT-specific settings
}
```

### 7. Python Blocks Runner (`apps/framework-cli/src/framework/python/wrappers/blocks_runner.py`)

**Changes Made:**
- ✅ Added `wait_end_of_query: 1` to DDL operations
- ✅ **Fixed settings override issue** by merging with default client settings
- ✅ Preserves existing client defaults like `date_time_input_format`

```python
# Merge with any existing client settings to preserve defaults
default_settings = getattr(ch_client, 'default_settings', {})
ch_client.command(query, settings={
    **default_settings, 
    'wait_end_of_query': 1
})
```

## 🧪 Testing Strategy

### Rust Tests
- ✅ Unit tests for `query_param()` function
- ✅ Verifies `wait_end_of_query` added for INSERT, CREATE, ALTER, DROP, TRUNCATE
- ✅ Verifies `wait_end_of_query` NOT added for SELECT, SHOW, DESCRIBE
- ✅ Tests case sensitivity and whitespace handling

### Python Tests
- ✅ Unit tests for `_prepare_insert_options()`
- ✅ Tests both array and stream insert operations
- ✅ Verifies settings propagation in retry scenarios
- ✅ Mock-based testing to avoid real ClickHouse dependency

## 📚 Documentation Sources

1. **ClickHouse HTTP Interface Documentation**: https://clickhouse.com/docs/interfaces/http
2. **ClickHouse Response Buffering**: https://clickhouse.com/docs/interfaces/http#response-buffering
3. **ClickHouse HTTP Response Codes**: https://clickhouse.com/docs/interfaces/http#http_response_codes_caveats
4. **ClickHouse Async Inserts**: https://clickhouse.com/docs/optimize/asynchronous-inserts

## ⚡ Performance Impact Analysis

### ✅ **Positive Impact**
- **INSERT Operations**: Guaranteed at least once delivery
- **DDL Operations**: Confirmed acknowledgment before proceeding
- **Migrations**: Safe sequential execution

### ✅ **No Negative Impact**
- **SELECT Operations**: Full streaming performance maintained
- **Query Concurrency**: Multiple SELECT queries run without blocking
- **Memory Usage**: No unnecessary response buffering
- **Latency**: SELECT results stream immediately

## 🔍 PR Comments Integration

Based on [PR #2529](https://github.com/514-labs/moose/pull/2529) feedback:

### 1. ✅ Comment Accuracy Fixed
- **Issue**: DDL acknowledgment mentioned in INSERT-only contexts
- **Solution**: Updated comments to accurately reflect operation type
  - INSERT contexts: "Ensure at least once delivery for INSERT operations"
  - DDL contexts: "Ensure at least once delivery and DDL acknowledgment"

### 2. ✅ DDL Commands Extracted to Constants
- **Issue**: Hardcoded DDL verbs in Rust client
- **Solution**: Created `DDL_COMMANDS` constant array for maintainability
- **Benefit**: Easy to add new DDL operations like TRUNCATE

### 3. ✅ Settings Override Fixed
- **Issue**: Python blocks runner could drop default client settings
- **Solution**: Merge with existing client defaults instead of overriding
- **Code**: `{**default_settings, 'wait_end_of_query': 1}`

### 4. ✅ Comprehensive Unit Tests Added
- **Rust**: Tests for `query_param()` with various query types
- **Python**: Tests for OlapTable insert operations and settings propagation
- **Coverage**: INSERT, DDL, and SELECT query scenarios

## 🎯 Summary

This corrected implementation ensures:

✅ **At least once delivery for INSERT operations** - No data loss  
✅ **DDL acknowledgment for schema changes** - Safe migrations  
✅ **Preserved SELECT query performance** - No response buffering  
✅ **Maintained concurrency** - SELECT queries don't block each other  
✅ **Comprehensive testing** - Unit tests verify correct behavior  
✅ **Maintainable code** - Constants and proper commenting  
✅ **Settings preservation** - No accidental override of client defaults  

The implementation now correctly balances reliability for INSERT/DDL operations with performance for SELECT operations.