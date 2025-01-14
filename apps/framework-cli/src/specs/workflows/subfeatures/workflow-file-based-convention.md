# File-Based Convention

## Overview
Moose uses an intuitive, file-based convention for defining workflows that makes it natural to organize, understand, and maintain your data pipelines. The directory structure and file naming patterns directly reflect your workflow's execution logic.

## Directory Structure

### Basic Layout
```
workflows/
├── daily-etl/                # Workflow name
│ ├── 1.extract.py           # Sequential step
│ ├── 2.transform.py         # Next step
│ ├── 3.parallel/            # Parallel execution
│ │ ├── process-a.py         # Runs in parallel
│ │ └── process-b.py         # Runs in parallel
│ └── 4.load.scala           # Final step
├── shared/                  # Shared utilities
│ └── utils.py              # Common code
└── config.toml             # Workflow config
```

## Naming Conventions

### Step Ordering
- Numeric prefixes define execution order: `1.extract.py`, `2.transform.py`
- Steps run sequentially by default
- Parallel execution through directory grouping
- File extensions determine language: `.py`, `.ts`, `.scala`

```python
# 1.extract.py
@task
def extract():
    return read_data()

# 2.transform.py
@task
def transform(data):
    return process_data(data)
```

### Parallel Execution
```
3.parallel/
├── process-a.py    # These files run
└── process-b.py    # in parallel
```

```python
# process-a.py
@task
def process_a(data):
    return transform_subset_a(data)

# process-b.py
@task
def process_b(data):
    return transform_subset_b(data)
```

## Feature Highlights

### Directory-Based Organization
- Workflows are self-contained directories
- Clear visual representation of flow
- Easy to understand execution order
- Natural parallel processing definition

### Shared Code Management
```
shared/
├── utils.py           # Common utilities
├── constants.py       # Shared constants
└── types/            # Shared type definitions
    ├── __init__.py
    └── models.py
```

### Configuration Files
```toml
# config.toml
[workflow]
name = "daily-etl"
schedule = "0 0 * * *"

[steps]
parallel_execution = true
timeout_per_step = "10m"

[dependencies]
shared = ["utils", "constants"]
```

## Benefits

### 1. Intuitive Organization
- Natural mapping of files to workflow steps
- Clear visualization of workflow structure
- Easy to understand execution order
- Simple parallel processing definition

### 2. Maintainability
- Each step is a separate file
- Clear separation of concerns
- Easy to modify individual steps
- Simple to add or remove steps

### 3. Code Reuse
- Shared code directory for common functionality
- Import utilities across workflows
- Consistent patterns across teams
- Reduced code duplication

### 4. Version Control Friendly
- Natural git workflow
- Easy to review changes
- Clear change history
- Simple conflict resolution

## Development Workflow

### Creating New Workflows
```bash
# Initialize new workflow
moose-cli workflow init my-workflow

# Generated structure
my-workflow/
├── 1.first-step.py
├── shared/
└── config.toml
```

### Adding Steps
```bash
# Add new step
moose-cli workflow add-step my-workflow "process-data"

# Add parallel steps
moose-cli workflow add-parallel my-workflow "process-subset"
```

### Templates
```bash
# Use workflow template
moose-cli workflow init my-workflow --template etl

# Custom templates
moose-cli workflow init my-workflow --template ./my-template
```

## Best Practices

### Directory Structure
1. Keep workflows focused and single-purpose
2. Use meaningful step names
3. Group related parallel steps
4. Maintain clean shared code

### Naming Conventions
1. Use descriptive step names
2. Follow consistent numbering
3. Group parallel steps logically
4. Use appropriate file extensions

### Code Organization
1. Leverage shared utilities
2. Keep steps focused
3. Use consistent patterns
4. Document complex flows

## Advanced Features

### Conditional Execution
```
workflows/
└── daily-etl/
    ├── 1.extract.py
    ├── 2.validate/
    │   ├── check.py
    │   └── skip-conditions.toml
    └── 3.load.py
```

### Step Dependencies
```toml
# step-config.toml
[dependencies]
requires = ["extract", "validate"]
optional = ["cleanup"]
```

### Dynamic Steps
```python
# dynamic-steps.py
@dynamic_task
def generate_steps(data):
    return [
        Step(f"process_{region}", process_fn)
        for region in data.regions
    ]
```