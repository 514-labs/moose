# Table Setup

## Overview
Tables in DMv2 are created using the `OlapTable` class, which provides a type-safe way to define your data infrastructure.

## Basic Table Configuration

```typescript
import { OlapTable, Key } from '@514labs/moose-lib';

// Basic table configuration
export const Example = new OlapTable("Example");
```

## Table Configuration Options

The `OlapTable` class accepts the following configuration options:

```typescript
type OlapConfig<T> = {
  orderByFields?: (keyof T & string)[];  // Fields to order by (must not be nullable)
  deduplicate?: boolean;                  // Whether to deduplicate records
};
```

### Key Requirements

When defining your schema, you must either:
1. Use `Key<T>` for one of the top-level fields, or
2. Specify the key field in `orderByFields`

Important requirements:
- If you use `Key<T>`, it must be the first field in `orderByFields` when specified
- Fields used in `orderByFields` must not be nullable (no optional fields or union with null)

### Basic Configuration Examples

```typescript
// Table with Key<T> field
interface KeyedSchema {
  id: Key<string>;
  name: string;
  value: number;
  createdAt: Date;      // Non-nullable field - can be used in orderByFields
  updatedAt?: Date;     // Nullable field - cannot be used in orderByFields
}

export const Keyed = new OlapTable("Keyed", {
  orderByFields: ["id", "createdAt"]  // Only non-nullable fields allowed
});

// ❌ Invalid: Cannot use nullable fields in orderByFields
export const InvalidKeyed = new OlapTable("InvalidKeyed", {
  orderByFields: ["id", "updatedAt"]  // Error: Cannot use nullable field 'updatedAt'
});

// Table with key specified in orderByFields
interface UnkeyedSchema {
  id: string;
  name: string;
  value: number;
}

export const Unkeyed = new OlapTable("Unkeyed", {
  orderByFields: ["id"]  // Key field must be non-nullable
});
```

## Table Examples

### Basic Table with Key
```typescript
// Simple schema with Key<T>
interface BasicSchema {
  id: Key<string>;
  name: string;
  value: number;
  active: boolean;
}

// Basic table with Key field
export const Basic = new OlapTable("Basic", {
  orderByFields: ["id"]  // Key field must be first
});
```

### Table with Ordering and Key
```typescript
// Schema with Key<T> and timestamp
interface TimeSeriesSchema {
  id: Key<string>;
  timestamp: Date;
  value: number;
  metadata: {
    source: string;
  };
}

// Table with Key field and ordering
export const TimeSeries = new OlapTable("TimeSeries", {
  orderByFields: ["id", "timestamp"]  // Key field must be first
});
```

### Table with Deduplication and Key
```typescript
// Schema with Key<T> and versioning
interface VersionedSchema {
  id: Key<string>;
  version: number;
  data: string;
  updatedAt: Date;
}

// Table that keeps only the latest version
export const Versioned = new OlapTable("Versioned", {
  deduplicate: true,
  orderByFields: ["id", "version", "updatedAt"]  // Key field must be first
});
```

## Best Practices

1. **Sorting Key Fields**:
   - Use only non-nullable fields in `orderByFields`
   - Make sure key fields are always populated
   - Consider using default values instead of optional fields for sorting keys

2. **Schema Design**:
   - Mark fields as optional (`?`) only when they truly can be missing
   - Use non-nullable fields for important indexing and sorting columns
   - Consider the query patterns when choosing sorting keys

## How It Works

When you create an `OlapTable` instance:
1. The table is registered in the global Moose registry
2. The schema is stored as JSON Schema (v3.1)
3. When deployed, Moose creates the corresponding infrastructure

## Development Workflow

### Local Development with Hot Reloading

One of the powerful features of DMv2 is its integration with the Moose development server:

1. Start your local development server with `moose dev`
2. When you define or modify an `OlapTable` in your code and save the file:
   - The changes are automatically detected
   - The TypeScript compiler plugin processes your schema definitions
   - The infrastructure is updated in real-time to match your code changes
   - Your tables are immediately available for testing

For example, if you add a new field to your schema:
```typescript
// Before
interface BasicSchema {
  id: Key<string>;
  name: string;
}

// After adding a field
interface BasicSchema {
  id: Key<string>;
  name: string;
  createdAt: Date;  // New field
}
```

The Moose framework will:
1. Detect the change when you save the file
2. Update the table schema in the local ClickHouse instance
3. Make the new field immediately available for use

### Verifying Your Tables

You can verify your tables were created correctly using:
```bash
# List all tables in your local environment
moose ls
```

Or by connecting directly to your local ClickHouse instance and running SQL commands. 