# MOOSE Scripts Orchestrator

## Context
Writing scripts and orchestrating them is a core part of the data engineering workflow from discovery, to build to maintenance and monitoring. Scripts are powerful in both ad-hoc data wrangling and in application-grade scenarios.

## High-Level Principles

1. **Language Agnostic**
   - Let the developer write scripts in whatever language they prefer
   - No compromise on native language features and tooling

2. **Works With Your Tools**
   - Support common data processing frameworks
   - Enable "works with your existing tools" value proposition
   - Seamless integration with pandas, spark, dask, etc.

3. **Developer Experience First**
   - Simple, simple developer ergonomics
   - Handle the 95% use cases really, really easily
   - Incrementally add complexity to handle the last 5%

## Overview
The MOOSE Scripts Orchestrator provides a file-based convention for defining and running workflow pipelines, supporting multiple programming languages while leveraging Temporal's reliability features under the hood.

## Key Features

### 1. File-Based Convention
Workflows are defined through directory structure and file naming:
```
workflows/
├── daily-etl/ # Workflow name (this is in py)
│ ├── 1.extract.py # Sequential step
│ ├── 2.transform.py # Different language
│ ├── 3.parallel/ # Parallel steps
│ │ ├── process-a.py
│ │ └── process-b.py
│ └── 4.load.scala
├── monthly-etl/ # Workflow name (this is in ts)
│ ├── 1.extract.ts
│ ├── 2.transform.ts
│ └── 3.load.ts
├── shared/ # Shared code
│ └── utils.py
└── config.toml # Workflow configuration
```

### 2. Multi-Language Support
- Native language type definitions (dataclasses, interfaces, case classes)
- Transparent type conversion between steps
- DataFrame support across languages
- Language-specific testing utilities
- One language per workflow is supported

Example in Python:
```python
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

Example in TypeScript:
```typescript
interface ProcessedData {
    source: string;
    recordCount: number;
    summary: Record<string, number>;
}

export const process = task<ExtractedData, ProcessedData>(
    async (data: ExtractedData) => {
        // DataFrame operations available through wrapper
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

### 3. Workflow Configuration
```toml
# Basic workflow configuration
name = "daily-etl"
schedule = "0 0 * * *"
retries = 3
timeout = "1h"

# Alert configuration
[alerts]
on_failure = ["slack-channel", "email-team"]

# Resource requirements
[resources]
memory = "4Gi"
cpu = 2

# Environment variables
[env]
POSTGRES_URL = "${POSTGRES_URL}"
```

### 4. Developer Experience

#### CLI Commands

Create new workflow
```bash
moose workflow new daily-etl
```

Run workflow locally
```bash
moose workflow run daily-etl
```

Resume failed workflow from specific step
```bash
moose workflow resume daily-etl --from 2.transform
```

Run workflow with specific data
```bash
moose workflow run daily-etl --input-file ./test-data.json
```

Watch workflow during development
```bash
moose workflow dev daily-etl
```

Deploy workflow
```bash
moose workflow deploy daily-etl --env prod
```

List workflow status
```bash
moose workflow list
moose workflow status daily-etl
```

#### Local Development
- Hot reload during development
- Local testing with sample data
- Step-by-step debugging
- Type checking across language boundaries
- Automatic code generation for type bridges

### 5. Reliability Features

#### Idempotency
- Automatic deduplication of workflow runs
- Checkpointing of DataFrame operations
- State management through Temporal

#### Restartability
- Resume from any step
- Maintain state and data lineage
- Partial retries for failed steps

#### Monitoring
- Step-level metrics
- DataFrame operation statistics
- Memory and CPU usage
- Duration tracking
- Error reporting

## Technical Implementation

### 1. Type Bridge System
Uses Protocol Buffers as intermediate representation:

```protobuf
message DataFrameMetadata {
    string schema = 1;
    int64 num_rows = 2;
    repeated string column_names = 3;
}

message TaskInput {
    bytes data = 1;
    DataFrameMetadata df_metadata = 2;
    string type_name = 3;
}

message TaskOutput {
    bytes data = 1;
    DataFrameMetadata df_metadata = 2;
    string type_name = 3;
}
```

### 2. Temporal Integration
- Each workflow maps to a Temporal workflow
- Steps map to Temporal activities
- Parallel directories become parallel activity execution
- File naming determines execution order
- Configuration maps to Temporal workflow options

### 3. DataFrame Handling
- Apache Arrow as intermediate format
- Lazy evaluation where possible
- Optimization of operations across steps
- Automatic persistence for large DataFrames

## Error Handling

### 1. Step Failures
- Automatic retry with backoff
- Maintain partial results
- Option to skip failed steps
- Detailed error reporting

### 2. Data Validation
- Schema validation between steps
- DataFrame integrity checks
- Type compatibility verification
- Size and resource limits

## Security

### 1. Data Protection
- Encryption at rest for persisted DataFrames
- Secure credential handling
- Access control per workflow

### 2. Audit
- Detailed execution logs
- Data lineage tracking
- Access logs
- Configuration change history

## Future Considerations

1. **Workflow Templates**
   - Common patterns as templates
   - Shareable workflow definitions
   - Template marketplace

2. **Advanced Features**
   - Workflow versioning
   - A/B testing support
   - Automatic documentation
   - Performance optimization suggestions

3. **Integration Ecosystem**
   - Common data source connectors
   - Cloud service integrations
   - Monitoring system plugins

4. **Community and Ecosystem**
   - Shared libraries and packages
   - User-contributed templates and workflows
   - Developer forums and support
