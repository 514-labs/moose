# MOOSE Workflow Orchestrator

## Context
Writing scripts and orchestrating them is a core part of the data engineering workflow from discovery, to build to maintenance and monitoring. MOOSE Workflow Orchestrator provides a comprehensive solution that works with your existing tools and practices while adding enterprise-grade reliability and a simple, intuitive developer experience.

## High-Level Principles

1. **Language Agnostic**
   - Write workflows in Python, TypeScript, Scala, or other supported languages
   - Native language features and tooling preserved
   - Type-safe communication between language boundaries

2. **Works With Your Tools**
   - Seamless integration with both big and small data frameworks
   - Automatic framework selection based on data scale
   - Support for pandas, polars, PySpark, DuckDB, and more

3. **Developer Experience First**
   - Intuitive file-based workflow organization
   - Comprehensive CLI and development tools
   - Hot reload and debugging support

## Core Features

### File-Based Convention
Intuitive directory structure for workflow organization:
```
workflows/
├── daily-etl/               # Workflow name
│ ├── 1.extract.py           # Sequential step
│ ├── 2.transform.py         # Next step
│ ├── 3.parallel/            # Parallel execution
│ │ ├── process-a.py         # Runs in parallel
│ │ └── process-b.py         # Runs in parallel
│ ├── 4.load.scala           # Next step
| └── 5.send-report          # Final step
|   └── child.toml           # Child workflow configuration
├── send-report/             # Workflow name
│ ├── 1.prepare.py
│ ├── 2.package.py
│ └── 3.send.py
├── monthly-etl/ # Workflow name (this is in ts)
│ ├── 1.extract.ts
│ ├── 2.transform.ts
│ └── 3.load.ts
├── shared/                  # Shared utilities
│ └── utils.py               # Common code
└── config.toml              # Workflow config
```

#### Parallel Execution execution

#### Workflow composition
Enable workflows to be composed with other workflows

This is done by referencing the workflow name as a step in the workflow.

### Multi-Language Support
Native language support with type safety:
```python
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

### Big and Small Data Framework Supportworkflow-big-and-small-data-support.md)
Seamless support for both big and small data frameworks with automatic framework selection:

```python
# Framework-specific processing with type safety
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
```


Scale-based configuration:
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

Framework selection guide:
- Small Data (<1GB): Pandas for rich functionality and rapid development
- Medium Data (1-10GB): Polars/DuckDB for fast processing
- Large Data (>10GB): PySpark for distributed computing

### Workflow Configuration
Powerful TOML-based configuration:
```toml
# Basic workflow configuration
name = "daily-etl"
schedule = "0 0 * * *"
retries = 3
timeout = "1h"

[data.processing]
auto_scale = true
small_data_threshold = "1GB"
medium_data_threshold = "10GB"

[resources]
auto_allocate = true
min_memory = "1Gi"
max_memory = "64Gi"
```

### Developer Experience
Comprehensive development tools:
```bash
# Initialize and develop
moose-cli workflow init <NAME>
moose-cli workflow init <NAME> --steps=<STEP_NAME>,<STEP_NAME>,<STEP_NAME>
moose-cli workflow init <NAME> --step <STEP_NAME> --step <STEP_NAME> --step <STEP_NAME>

# Testing and debugging
moose-cli workflow run <NAME> --input <FILE>
moose-cli workflow resume <NAME> --from <STEP>
```

### Reliability Features
Enterprise-grade reliability:
```toml
[monitoring]
metrics_endpoint = "prometheus"
trace_sampling = 0.1
log_level = "info"

[alerts]
on_failure = ["slack", "email"]
on_duration_threshold = "1h"
```

## Technical Implementation

### Type System
- Protocol Buffers for cross-language type safety
- Apache Arrow for DataFrame operations
- Automatic type conversion and validation
- Schema compatibility checking

### Framework Integration
- Optimized data transfer between frameworks
- Resource-aware execution planning
- Framework-specific optimizations

### Temporal Backend
- Reliable workflow execution
- State management and checkpointing
- Activity retry and timeout handling
- Parallel execution coordination

### Security and Compliance
- Role-based access control
- Audit logging and tracking
- Secure credential management
- Data encryption at rest

## Best Practices

### Workflow Organization
1. Use meaningful step names and clear directory structure
2. Leverage parallel processing for independent operations
3. Share common code through the shared directory
4. Configure appropriate resource limits

### Framework Selection
1. Use pandas for small data and rapid development
2. Choose polars/DuckDB for medium-sized data
3. Leverage PySpark for large-scale processing
4. Enable auto-scaling for varying data sizes

### Development Workflow
1. Start with local development mode
2. Use test data for validation
3. Leverage hot reload for quick iterations
4. Monitor performance metrics

## Future Roadmap

### 1. Enhanced Framework Support
- Additional data processing frameworks
- Framework-specific optimizations
- Custom framework integration API

### 2. Advanced Features
- Workflow versioning and A/B testing
- Machine learning pipeline support
- Real-time workflow updates
- Advanced monitoring and analytics

### 3. Developer Tools
- Visual workflow designer
- Performance optimization suggestions
- Advanced debugging capabilities
- Workflow testing frameworks

### 4. Enterprise Features
- Multi-tenant support
- Advanced compliance controls
- Custom security integrations
- Enhanced monitoring and alerting
