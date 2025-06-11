# Schema Definition

## Overview

DMv2 provides a type-safe approach to defining your data schemas using TypeScript interfaces. These interfaces can be exported and reused across your project, ensuring type consistency throughout your codebase.

## Basic Schema Definition

```typescript
// Define your schema interface
export interface ExampleSchema {
  id: Key<string>; // Primary key field with underlying type
  name: string; // Required string field
  value: number; // Required numeric field
}

// You can now import and use this schema elsewhere
```

## Supported Types

```JSON
{
  "supportedTypes": [
    {
      "typescriptType": "string",
      "clickhouseType": "String",
      "supported": true
    },
    {
      "typescriptType": "number",
      "clickhouseType": "Float64",
      "supported": true
    },
    {
      "typescriptType": "boolean",
      "clickhouseType": "Boolean",
      "supported": true
    },
    {
      "typescriptType": "Date",
      "clickhouseType": "DateTime",
      "supported": true
    },
    {
      "typescriptType": "Object",
      "clickhouseType": "Nested",
      "supported": true
    },
    {
      "typescriptType": "Array",
      "clickhouseType": "Array",
      "supported": true
    },
    {
      "typescriptType": "T?",
      "clickhouseType": "Nullable",
      "supported": true
    },
    {
      "typescriptType": "Enum",
      "clickhouseType": "Enum",
      "supported": true
    },
    {
      "typescriptType": "Key<T>",
      "clickhouseType": "Same as T",
      "supported": true
    },
    {
      "typescriptType": "`T | null`",
      "clickhouseType": "Nullable(T)",
      "supported": true
    }
  ]
}
```

## Unsupported Types

The following TypeScript types are not supported:

- any
- Union types (e.g., `string | number`)
- `undefined`
- `null` as direct type
- `symbol`
- `bigint`
- Complex TypeScript types (tuples, mapped types, etc.)
- Those not listed in the table

## Workarounds for Unsupported Types

### Union Types

Union types with non-null types are not supported. However, union types with `null` are supported and automatically map to ClickHouse's Nullable type.

```typescript
// SUPPORTED: Nullable types
export interface SupportedSchema {
  id: Key<string>; // Key must specify its underlying type
  value1?: string; // ✅ Maps to Nullable(String)
  value2: string | null; // ✅ Also maps to Nullable(String)
}

// NOT SUPPORTED: Union of non-null types
export interface IncorrectSchema {
  id: Key;
  value: string | number; // ❌ This won't work
}

// CORRECT: Choose a single type
export interface CorrectSchema {
  id: Key;
  value: string; // ✅ Choose a single type that makes sense
}
```

### Complex TypeScript Types

For complex TypeScript types, simplify your schema by using basic types:

```typescript
// INCORRECT: Using complex types
export interface IncorrectSchema {
  id: Key<string>;
  coordinates: [number, number]; // ❌ Tuple not supported
  metadata: Record<string, string>; // ❌ Complex mapped type
}

// CORRECT: Use nested objects and arrays
export interface CorrectSchema {
  id: Key<string>;
  coordinates: { lat: number; lng: number }; // ✅ Standard object
  metadata: { key: string; value: string }[]; // ✅ Array of objects
}
```

### BigInt and Symbol

For large integers that would normally use `bigint`, use `string` to preserve precision:

```typescript
// INCORRECT: Using BigInt
export interface IncorrectSchema {
  id: Key<string>;
  largeNumber: bigint; // ❌ Not supported
}

// CORRECT: Store as string
export interface CorrectSchema {
  id: Key<string>;
  largeNumber: string; // ✅ Store large numbers as strings
}
```

## Best Practices

1. **Define clear schemas**: Use descriptive field names and appropriate types
2. **Document your models**: Add comments to explain the purpose of fields
3. **Use consistent naming**: Adopt a convention for tables and fields
4. **Organize related models**: Group related schemas in dedicated files and export them
5. **Version your schemas**: Plan for schema evolution in production

## Schema Validation

DMv2 validates your schemas at different stages:

1. **Development-time validation**: TypeScript ensures your schemas are type-safe
2. **Compile-time validation**: The compiler plugin verifies that your schemas are valid
3. **Runtime validation**: When your code changes are hot-reloaded, Moose validates that your schemas can be properly mapped to ClickHouse tables
4. **Query validation**: Queries can be validated against your data models to ensure type safety and consistency

### Query Validation Against Data Models

When using consumption APIs, you can validate your queries against your data models:

```typescript
// Define your data model
export interface UserAnalytics {
  id: Key<string>;
  userId: string;
  eventCount: number;
  totalValue: number;
  timestamp: Date;
}

// Create a consumption API with data model validation
export const UserAnalyticsApi = new ConsumptionApi<QueryParams>(
  "user-analytics",
  async (params, utils) => {
    const { client, sql } = utils;
    return await client.query.execute(sql`
      SELECT 
        id,
        user_id as userId,
        COUNT(*) as eventCount,
        SUM(value) as totalValue,
        MAX(timestamp) as timestamp
      FROM events
      WHERE user_id = ${params.userId}
      GROUP BY id, user_id
    `);
  },
  {
    metadata: {
      description: "User analytics with data model validation",
    },
    validation: {
      enabled: true,
      dataModel: "UserAnalytics",
      validateAgainstModel: true,
    },
  }
);
```

The validation will:

1. Check that all required fields from the data model are present in the query results
2. Verify that field types match the data model
3. Report any mismatches or missing fields
4. Ensure type safety between your TypeScript interfaces and ClickHouse tables

If there are issues with your schema:

- TypeScript errors will appear in your IDE for type-related issues
- Console errors will show in the development server output for schema problems
- Runtime errors will be displayed if there are issues applying your changes to the database
- Query validation errors will show if your query results don't match your data model
