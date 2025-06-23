# Consumption APIs

## Overview

Consumption APIs provide a type-safe way to query and transform data from your tables. They allow you to create custom endpoints that can perform complex queries, transformations, and data processing.

## Basic Consumption API Setup

```typescript
import { ConsumptionApi } from "@514labs/moose-lib";
import { tags } from "typia";

// Define your parameter interface with Typia tags for validation
interface QueryParams {
  userId: string & tags.Format<"uuid">; // Must be a valid UUID
  startDate: Date;
  endDate: Date;
  limit?: number & tags.Type<"int64"> & tags.Minimum<1> & tags.Maximum<100>;
}

// Create a consumption API
export const UserAnalyticsApi = new ConsumptionApi<QueryParams>(
  "user-analytics",
  async (params, utils) => {
    const { client, sql } = utils;

    // Use the SQL helper for type-safe queries
    const result = await client.query.execute(sql`
      SELECT 
        user_id,
        COUNT(*) as event_count,
        SUM(value) as total_value
      FROM events
      WHERE user_id = ${params.userId}
        AND timestamp BETWEEN ${params.startDate} AND ${params.endDate}
      GROUP BY user_id
      LIMIT ${params.limit ?? 10}
    `);

    return result;
  },
  {
    metadata: {
      description: "User analytics egress api",
    },
  }
);
```

## Aurora MCP Query Validation (Development Only)

Aurora MCP provides query validation during API development to ensure your APIs are reliable. This validation happens when you create APIs with Aurora MCP, not at runtime in your Moose application.

**Important**: The validation configuration is NOT part of the Moose library API - it's a development-time feature of Aurora MCP.

### How Aurora MCP Validation Works

1. **During API Generation**: Aurora MCP tests your SQL query against ClickHouse
2. **Development Feedback**: You get immediate feedback if queries fail
3. **Generated Code**: The final API code contains no validation configuration

## Parameter Validation with Typia

Typia provides powerful type validation capabilities for your query parameters. Here are some common use cases:

```typescript
import { tags } from "typia";

interface ValidatedParams {
  // String validations
  email: string & tags.Format<"email">;
  uuid: string & tags.Format<"uuid">;
  url: string & tags.Format<"url">;

  // Number validations
  positiveInt: number & tags.Type<"int64"> & tags.Minimum<1>;
  percentage: number & tags.Type<"int64"> & tags.Minimum<0> & tags.Maximum<100>;

  // Date validations
  futureDate: Date & tags.ExclusiveMinimum<Date.now()>;

  // Enum-like validations
  status: "active" | "inactive" | "pending";

  // Optional fields with validation
  optionalLimit?: number & tags.Type<"int64"> & tags.Minimum<1>;
}
```

## Handling Optional Parameters (CRITICAL for TypeScript)

**⚠️ Common TypeScript Error**: Passing optional parameters (`undefined` values) directly to SQL template literals causes compilation errors:

```typescript
// ❌ WRONG: This causes TypeScript compilation errors
interface BadParams {
  userId: string;
  startDate?: string; // Can be undefined
  endDate?: string; // Can be undefined
}

export const BadApi = new ConsumptionApi<BadParams>(
  "bad-example",
  async (params, { client, sql }) => {
    // ❌ TypeScript Error: Argument of type 'string | undefined' is not assignable
    const result = await client.query.execute(sql`
      SELECT * FROM events 
      WHERE user_id = ${params.userId}
        AND timestamp >= ${params.startDate}  -- ❌ Can be undefined!
        AND timestamp <= ${params.endDate}    -- ❌ Can be undefined!
    `);
    return result;
  }
);
```

### ✅ **Robust Solution: SQL Conditional Logic with Default Values**

The best approach uses SQL conditional logic with default values to handle optional parameters:

```typescript
interface RobustParams {
  userId: string;
  startDate?: string; // Optional ISO date string
  endDate?: string; // Optional ISO date string
  limit?: number; // Optional limit
}

export const RobustApi = new ConsumptionApi<RobustParams>(
  "robust-example",
  async (params, { client, sql }) => {
    // ✅ CORRECT: Use default values with SQL conditional logic
    const result = await client.query.execute(sql`
      SELECT 
        user_id,
        event_type,
        timestamp,
        value
      FROM events 
      WHERE user_id = ${params.userId}
        -- Optional start date filter
        AND (${params.startDate || "1900-01-01"} = '1900-01-01' 
             OR timestamp >= ${params.startDate || "1900-01-01"})
        -- Optional end date filter  
        AND (${params.endDate || "2099-12-31"} = '2099-12-31'
             OR timestamp <= ${params.endDate || "2099-12-31"})
      ORDER BY timestamp DESC
      LIMIT ${params.limit || 100}
    `);
    return result;
  }
);
```

### **Why This Pattern Works:**

1. **No undefined values**: `|| 'default'` ensures TypeScript never sees `undefined`
2. **SQL conditional logic**: When no parameter is provided, the condition `'1900-01-01' = '1900-01-01'` evaluates to `TRUE`, effectively ignoring that filter
3. **Type safety**: All values passed to SQL are guaranteed to be strings/numbers
4. **Performance**: ClickHouse optimizes these simple conditions efficiently

### **Alternative Patterns (Less Robust):**

```typescript
// ❌ AVOID: Pre-processing with separate variables (verbose and error-prone)
export const VerboseApi = new ConsumptionApi<MyParams>(
  "verbose-example",
  async (params, { client, sql }) => {
    const startDate = params.startDate || "1900-01-01";
    const endDate = params.endDate || "2099-12-31";

    // Still need conditional logic for truly optional filters
    const result = await client.query.execute(sql`
      SELECT * FROM events 
      WHERE user_id = ${params.userId}
        AND timestamp >= ${startDate}
        AND timestamp <= ${endDate}
    `);
    return result;
  }
);

// ❌ AVOID: Dynamic query building (complex and unmaintainable)
export const DynamicApi = new ConsumptionApi<MyParams>(
  "dynamic-example",
  async (params, { client, sql }) => {
    let whereClause = `user_id = ${params.userId}`;
    if (params.startDate) {
      whereClause += ` AND timestamp >= ${params.startDate}`;
    }
    if (params.endDate) {
      whereClause += ` AND timestamp <= ${params.endDate}`;
    }

    // ❌ This approach is complex and harder to maintain
    const result = await client.query.execute(sql`
      SELECT * FROM events WHERE ${whereClause}
    `);
    return result;
  }
);
```

### **More Examples of the Robust Pattern:**

```typescript
// Multiple optional filters
interface AdvancedParams {
  userId?: string; // Optional user filter
  eventType?: string; // Optional event type filter
  minValue?: number; // Optional minimum value
  maxValue?: number; // Optional maximum value
  startDate?: string; // Optional date range start
  endDate?: string; // Optional date range end
  limit?: number; // Optional result limit
}

export const AdvancedFilterApi = new ConsumptionApi<AdvancedParams>(
  "advanced-filters",
  async (params, { client, sql }) => {
    const result = await client.query.execute(sql`
      SELECT 
        user_id,
        event_type,
        value,
        timestamp
      FROM events 
      WHERE 1=1  -- Always true, makes AND conditions easier
        -- Optional user filter
        AND (${params.userId || ""} = '' OR user_id = ${params.userId || ""})
        -- Optional event type filter
        AND (${params.eventType || ""} = '' OR event_type = ${params.eventType || ""})
        -- Optional value range filters
        AND (${params.minValue || -999999} = -999999 OR value >= ${params.minValue || -999999})
        AND (${params.maxValue || 999999} = 999999 OR value <= ${params.maxValue || 999999})
        -- Optional date range filters  
        AND (${params.startDate || "1900-01-01"} = '1900-01-01' 
             OR timestamp >= ${params.startDate || "1900-01-01"})
        AND (${params.endDate || "2099-12-31"} = '2099-12-31'
             OR timestamp <= ${params.endDate || "2099-12-31"})
      ORDER BY timestamp DESC
      LIMIT ${params.limit || 100}
    `);
    return result;
  }
);
```

### Available Utilities

The `ConsumptionUtil` object provides several helpful utilities:

```typescript
interface ConsumptionUtil {
  client: MooseClient; // Query client for database access
  sql: Sql; // SQL template literal helper
  jwt?: any; // JWT payload if authentication is enabled
}
```

### SQL Helper

The `sql` template literal helper provides type-safe SQL query construction. For dynamic column or table names, use the `ConsumptionHelpers`:

```typescript
import { ConsumptionHelpers as CH } from "@514labs/moose-lib";

// Use static strings directly - no need for ConsumptionHelpers
const staticQuery = sql`
  SELECT *
  FROM events
  WHERE user_id = ${userId}
`;

// Example of dynamic column ordering and filtering
interface DynamicQueryParams {
  columns: Array<"user_id" | "timestamp" | "event_type" | "value">;
  filters: Array<{
    column: "user_id" | "timestamp" | "event_type" | "value";
    operator: "=" | ">" | "<" | ">=" | "<=" | "!=" | "LIKE";
    value: string | number | Date;
  }>;
  orderBy?: {
    column: "user_id" | "timestamp" | "event_type" | "value";
    direction: "ASC" | "DESC";
  };
}

export const DynamicQueryApi = new ConsumptionApi<DynamicQueryParams>(
  "dynamic-query",
  async (params, utils) => {
    const { client, sql } = utils;

    // Build dynamic column list
    const columnList = params.columns.map((col) => CH.column(col)).join(", ");

    // Build dynamic filters
    const filterConditions = params.filters.map(
      (filter) => sql`${CH.column(filter.column)} ${filter.operator} ${filter.value}`
    );

    // Build dynamic order by
    const orderByClause = params.orderBy
      ? sql`ORDER BY ${CH.column(params.orderBy.column)} ${params.orderBy.direction}`
      : sql``;

    // Join filter conditions with AND
    const whereClause =
      filterConditions.length > 0
        ? sql`WHERE ${filterConditions.reduce((acc, curr, idx) =>
            idx === 0 ? curr : sql`${acc} AND ${curr}`
          )}`
        : sql``;

    return await client.query.execute(sql`
      SELECT ${sql`${columnList}`}
      FROM events
      ${whereClause}
      ${orderByClause}
    `);
  },
  {
    metadata: {
      description: "Dynamic query egress api example",
    },
  }
);
```

### Query Client

The `MooseClient` provides methods for executing queries and managing workflows:

```typescript
// CORRECT: Execute a query and handle the ResultSet
const result = await client.query.execute(sql`...`);
const data = await result.json(); // Process the data if needed

// Start a workflow
const workflow = await client.workflow.execute("workflow-name", inputData);
```

**IMPORTANT**: For DMV2 (DuckDB) projects, always use `client.query.execute()` for database queries:

```typescript
// CORRECT: Use client.query.execute() for DMV2 projects
export const getBrainBySessionIdApi = new ConsumptionApi<QueryParams, Brain[]>(
  "get-brain-by-session-id",
  async ({ sessionId }, { client, sql }) => {
    if (!sessionId) {
      throw new Error("Missing required parameter: sessionId");
    }

    const result = await client.query.execute(
      sql`SELECT * FROM Brain WHERE sessionId = ${sessionId}`
    );
    return await result.json();
  }
);

// INCORRECT: Never use client.query alone
export const getBrainBySessionIdApi = new ConsumptionApi<QueryParams, Brain[]>(
  "get-brain-by-session-id",
  async ({ sessionId }, { client, sql }) => {
    // ❌ WRONG: This will not work in DMV2 projects
    const result = await client.query(sql`
      SELECT * FROM Brain WHERE sessionId = ${sessionId}
    `);
    return await result.json();
  }
);
```

## Examples

### Basic Query API with Optional Parameters

```typescript
import { tags } from "typia";

interface SimpleQueryParams {
  limit: number & tags.Type<"int64"> & tags.Minimum<1> & tags.Maximum<100>;
  offset?: number & tags.Type<"int64"> & tags.Minimum<0>;
  category?: string; // Optional filter
}

export const RecentEventsApi = new ConsumptionApi<SimpleQueryParams>(
  "recent-events",
  async (params, utils) => {
    const { client, sql } = utils;

    // ✅ CORRECT: Use robust optional parameter pattern
    return await client.query.execute(sql`
      SELECT *
      FROM events
      WHERE 1=1
        -- Optional category filter
        AND (${params.category || ""} = '' OR category = ${params.category || ""})
      ORDER BY timestamp DESC
      LIMIT ${params.limit}
      OFFSET ${params.offset || 0}
    `);
  },
  {
    metadata: {
      description: "Returns recent events with optional filters",
    },
  }
);
```

## Complete Example with Robust Optional Parameters

Here's a complete example showing the recommended approach for handling optional parameters:

```typescript
import { ConsumptionApi } from "@514labs/moose-lib";
import { tags } from "typia";

// Define your parameter interface with Typia validation
interface UserMetricsParams {
  userId: string & tags.Format<"uuid">;
  startDate?: string; // Optional ISO date string
  endDate?: string; // Optional ISO date string
  eventType?: string; // Optional event type filter
  limit?: number & tags.Type<"int64"> & tags.Minimum<1> & tags.Maximum<100>;
}

// Create a consumption API using the robust optional parameter pattern
export const UserMetricsApi = new ConsumptionApi<UserMetricsParams>(
  "user-metrics",
  async (params, utils) => {
    const { client, sql } = utils;

    // ✅ CORRECT: Use SQL conditional logic with default values
    const result = await client.query.execute(sql`
      SELECT
        user_id,
        event_type,
        COUNT(*) as event_count,
        SUM(value) as total_value,
        AVG(value) as average_value,
        MIN(timestamp) as start_time,
        MAX(timestamp) as end_time
      FROM events
      WHERE user_id = ${params.userId}
        -- Optional date range filters using robust pattern
        AND (${params.startDate || "1900-01-01"} = '1900-01-01' 
             OR timestamp >= ${params.startDate || "1900-01-01"})
        AND (${params.endDate || "2099-12-31"} = '2099-12-31'
             OR timestamp <= ${params.endDate || "2099-12-31"})
        -- Optional event type filter
        AND (${params.eventType || ""} = '' OR event_type = ${params.eventType || ""})
      GROUP BY user_id, event_type
      ORDER BY event_count DESC
      LIMIT ${params.limit || 10}
    `);

    return result;
  },
  {
    metadata: {
      description: "User metrics API with robust optional parameter handling",
    },
  }
);

// Example usage
async function getUserMetrics(
  userId: string,
  filters?: {
    startDate?: string;
    endDate?: string;
    eventType?: string;
  }
) {
  const params: UserMetricsParams = {
    userId,
    ...filters,
    limit: 50,
  };

  try {
    const response = await fetch("/api/user-metrics", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(params),
    });

    if (!response.ok) {
      const error = await response.json();
      console.error("Failed to get user metrics:", error);
      return null;
    }

    return await response.json();
  } catch (error) {
    console.error("Error getting user metrics:", error);
    return null;
  }
}
```

## Key Takeaways for TypeScript Optional Parameters

1. **Always use default values**: Never pass `undefined` to SQL template literals
2. **Use SQL conditional logic**: Let the database handle optional filters efficiently
3. **Choose appropriate defaults**: Use extreme dates (`1900-01-01`, `2099-12-31`) for date ranges, empty strings for text filters, extreme numbers for numeric ranges
4. **Test thoroughly**: Verify your APIs work with all combinations of optional/required parameters
5. **Keep it simple**: Avoid complex dynamic query building - the conditional SQL pattern is cleaner and more maintainable

## Troubleshooting Common TypeScript Compilation Errors

### Error: "Argument of type 'string | undefined' is not assignable"

**Problem**: This error occurs when optional parameters are passed directly to SQL template literals:

```typescript
// ❌ This code will cause TypeScript compilation errors
interface HeartRateParams {
  user_id: string;
  start_date?: string; // Optional parameter
  end_date?: string; // Optional parameter
}

export const getHeartRateStats = new ConsumptionApi<HeartRateParams>(
  "getHeartRateStats",
  async (params, { client, sql }) => {
    // ❌ TypeScript Error: Argument of type 'string | undefined' is not assignable
    const result = await client.query.execute(sql`
      SELECT 
        user_id,
        COUNT(*) as measurement_count,
        MIN(heart_rate) as min_heart_rate,
        MAX(heart_rate) as max_heart_rate,
        AVG(heart_rate) as avg_heart_rate
      FROM heart_rate_measurement 
      WHERE user_id = ${params.user_id}
        AND timestamp >= ${params.start_date}  -- ❌ Can be undefined!
        AND timestamp <= ${params.end_date}    -- ❌ Can be undefined!
      GROUP BY user_id
    `);
    return result;
  }
);
```

**Solution**: Use the robust conditional SQL pattern with default values:

```typescript
// ✅ This code compiles and works correctly
interface HeartRateParams {
  user_id: string;
  start_date?: string; // Optional parameter
  end_date?: string; // Optional parameter
}

export const getHeartRateStats = new ConsumptionApi<HeartRateParams>(
  "getHeartRateStats",
  async (params, { client, sql }) => {
    // ✅ CORRECT: Use robust conditional SQL pattern
    const result = await client.query.execute(sql`
      SELECT 
        user_id,
        COUNT(*) as measurement_count,
        MIN(heart_rate) as min_heart_rate,
        MAX(heart_rate) as max_heart_rate,
        AVG(heart_rate) as avg_heart_rate,
        MIN(timestamp) as first_measurement,
        MAX(timestamp) as last_measurement
      FROM heart_rate_measurement 
      WHERE user_id = ${params.user_id}
        -- Robust optional date filters
        AND (${params.start_date || "1900-01-01"} = '1900-01-01' 
             OR timestamp >= ${params.start_date || "1900-01-01"})
        AND (${params.end_date || "2099-12-31"} = '2099-12-31' 
             OR timestamp <= ${params.end_date || "2099-12-31"})
      GROUP BY user_id
    `);
    return result;
  }
);
```

**Why This Works**:

- `params.start_date || '1900-01-01'` ensures TypeScript never sees `undefined`
- When `start_date` is not provided, `'1900-01-01' = '1900-01-01'` evaluates to `TRUE`, making the entire condition pass
- When `start_date` is provided, the actual filter `timestamp >= start_date` applies
- The database efficiently optimizes these simple conditional expressions

### Testing the Fix

You can test that your optional parameters work correctly:

```typescript
// Test 1: All parameters provided
const fullQuery = {
  user_id: "123e4567-e89b-12d3-a456-426614174000",
  start_date: "2024-01-01T00:00:00Z",
  end_date: "2024-12-31T23:59:59Z",
};

// Test 2: Only required parameters
const minimalQuery = {
  user_id: "123e4567-e89b-12d3-a456-426614174000",
  // start_date and end_date are undefined
};

// Test 3: Partial optional parameters
const partialQuery = {
  user_id: "123e4567-e89b-12d3-a456-426614174000",
  start_date: "2024-06-01T00:00:00Z",
  // end_date is undefined
};

// All three should compile and work correctly with the robust pattern
```
