# Multi-Language Support

## Overview
True multi-language support that enables developers to write workflow steps in their preferred programming language while maintaining type safety and DataFrame compatibility across language boundaries.

## Key Features

### Native Language Features
- Use native language type definitions (dataclasses, interfaces, case classes)
- Full access to language-specific testing frameworks
- Native package management and tooling support
- One language per workflow support for consistency

### DataFrame Operations
- Seamless integration with pandas, spark, and dask
- Transparent DataFrame conversion between languages
- Consistent DataFrame API across language boundaries
- Optimized performance for large datasets

### Type System
```python
# Python Example
from dataclasses import dataclass
from moose_lib import task
import pandas as pd

@dataclass
class ExtractedData:
    source: str
    timestamp: datetime
    records: pd.DataFrame

@task
def extract() -> ExtractedData:
    df = pd.read_csv("source.csv")
    return ExtractedData(
        source="sales_db",
        timestamp=datetime.now(),
        records=df
    )
```

```typescript
// TypeScript Example
interface ProcessedData {
    source: string;
    recordCount: number;
    summary: Record<string, number>;
}

export const process = task<ExtractedData, ProcessedData>(
    async (data: ExtractedData) => {
        const summary = await data.records
            .groupBy('category')
            .agg(['sum', 'count'])
        
        return {
            source: data.source,
            recordCount: data.records.length,
            summary: summary
        }
    }
);
```

## Technical Details

### Type Bridge System
- Protocol Buffers as intermediate representation
- Automatic type conversion between languages
- Schema validation and compatibility checks
- Support for complex nested types

### DataFrame Handling
- Apache Arrow as intermediate format
- Lazy evaluation where possible
- Optimized operations across language boundaries
- Automatic memory management