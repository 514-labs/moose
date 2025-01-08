# Big and Small Data Framework Support

## Overview
MOOSE workflow orchestration is designed to work seamlessly with both big data and small data frameworks, allowing developers to choose the right tool for their data scale without changing their workflow patterns.

## Supported Frameworks

### Python Ecosystem
```python
from moose_lib import task
import pandas as pd
import polars as pl
import pyspark.sql as spark
import duckdb

@task
def process_with_pandas(data_path: str) -> pd.DataFrame:
    # Ideal for smaller datasets with rich functionality
    df = pd.read_csv(data_path)
    return df.groupby('category').agg({'amount': ['sum', 'mean']})

@task
def process_with_polars(data_path: str) -> pl.DataFrame:
    # Fast processing for medium-sized data
    df = pl.scan_csv(data_path)
    return df.groupby('category').agg([
        pl.col('amount').sum(),
        pl.col('amount').mean()
    ]).collect()

@task
def process_with_spark(data_path: str) -> spark.DataFrame:
    # Distributed processing for big data
    spark = SparkSession.builder.getOrCreate()
    df = spark.read.csv(data_path)
    return df.groupBy('category').agg({'amount': 'sum'})

@task
def process_with_duckdb(data_path: str) -> pd.DataFrame:
    # SQL-based analytics for medium to large data
    return duckdb.sql("""
        SELECT category,
               SUM(amount) as total,
               AVG(amount) as average
        FROM read_csv_auto(?)
        GROUP BY category
    """, [data_path]).df()
```

### TypeScript Ecosystem
```typescript
import { task } from '@moose/lib';
import { DataFrame as PlDataFrame } from 'nodejs-polars';
import { DuckDB } from 'duckdb-async';

export const processWithPolars = task<string, PlDataFrame>(
    async (dataPath: string) => {
        const df = await pl.readCSV(dataPath);
        return df.groupBy('category')
            .agg([
                pl.col('amount').sum(),
                pl.col('amount').mean()
            ]);
    }
);

export const processWithDuckDB = task<string, any>(
    async (dataPath: string) => {
        const db = await DuckDB.create();
        return await db.all(`
            SELECT category,
                   SUM(amount) as total,
                   AVG(amount) as average
            FROM read_csv_auto(?)
            GROUP BY category
        `, [dataPath]);
    }
);
```

## Key Features

### Scale-Based Optimization
```toml
[data.processing]
# Automatically switch processing framework based on data size
auto_scale = true
small_data_threshold = "1GB"
medium_data_threshold = "10GB"

[resources]
# Dynamic resource allocation
auto_allocate = true
min_memory = "1Gi"
max_memory = "64Gi"
```

### Framework Interoperability
- Frameowork agnostic DataFrames (using Ibis (python only))
- Preserve schema and data types
- Minimize memory overhead
- Optimize performance

## Use Cases

### Small Data Processing
- Interactive data analysis
- Quick prototyping
- Report generation
- Local development
```python
@task
def quick_analysis():
    # Pandas for rapid development
    df = pd.read_csv('sample.csv')
    return df.describe()
```

### Medium Data Processing
- Batch processing
- Data transformation
- Complex aggregations
- Memory-efficient operations
```python
@task
def medium_scale_processing():
    # Polars for fast processing
    df = pl.scan_csv('medium_data.csv')
    return df.lazy().groupby('key').agg([
        pl.col('value').sum(),
        pl.col('value').mean()
    ]).collect()
```

### Big Data Processing
- Distributed computation
- Large-scale ETL
- Historical analysis
- Production workloads
```python
@task
def big_data_processing():
    # PySpark for distributed processing
    spark = SparkSession.builder.getOrCreate()
    df = spark.read.parquet('s3://big-data-bucket/')
    return df.groupBy('key').agg(F.sum('value'))
```

## Performance Optimization

### Memory Management
- Lazy evaluation support
- Streaming processing
- Automatic garbage collection
- Memory-aware scheduling

### Execution Optimization
```toml
[optimization]
# Framework-specific optimizations
enable_lazy_evaluation = true
enable_parallel_processing = true
chunk_size = "256MB"
cache_strategy = "memory_and_disk"

[monitoring]
# Performance tracking
track_memory_usage = true
track_execution_time = true
track_data_throughput = true
```

## Framework Selection Guide

### Decision Factors
1. Data Size
   - Small (<1GB): Pandas
   - Medium (1-10GB): Polars/DuckDB
   - Large (>10GB): PySpark

2. Processing Requirements
   - Interactive Analysis: Pandas
   - Fast Batch Processing: Polars
   - Distributed Computing: PySpark
   - SQL Analytics: DuckDB

3. Development Speed
   - Rapid Prototyping: Pandas
   - Production Performance: Polars/PySpark
   - SQL Workflows: DuckDB