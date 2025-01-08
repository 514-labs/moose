# Developer Experience

## Overview
Comprehensive developer tooling and local development features that make workflow development intuitive and efficient, from initialization to deployment.

## CLI Commands

### Workflow Management
```bash
# Initialize new workflow
moose-cli workflow init <NAME>

# Run workflow
moose-cli workflow run <NAME>

# Development mode with hot reload
moose-cli workflow dev <NAME>

# Deploy workflow
moose-cli workflow deploy <NAME> --env <ENV>
```

### Testing and Debugging
```bash
# Run with test data
moose-cli workflow run <NAME> --input <FILE>

# Resume from specific step
moose-cli workflow resume <NAME> --from <STEP>

# View workflow status
moose-cli workflow status <NAME>

# List all workflows
moose-cli workflow ls
```

## Local Development Features

### Hot Reload
- Automatic detection of file changes
- Immediate workflow updates
- Maintains workflow state during development
- Quick iteration cycles

### Debugging
- Step-by-step debugging support
- Breakpoint functionality
- Variable inspection
- State examination at any point

### Testing
- Local testing with sample data
- Automated test generation
- Language-specific test utilities
- Integration test support

### Type Safety
- Cross-language type checking
- Automatic type bridge generation
- Real-time type validation
- IDE integration for type hints

## Project Structure
```
workflows/
├── daily-etl/          # Workflow directory
│ ├── 1.extract.py      # Sequential step
│ ├── 2.transform.py    # Transform step
│ ├── 3.parallel/       # Parallel steps
│ │ ├── process-a.py
│ │ └── process-b.py
│ └── 4.load.scala
├── shared/            # Shared utilities
│ └── utils.py
└── config.toml        # Configuration
```

## Development Tools
- IDE integration
- Code generation utilities
- Documentation generation
- Performance profiling tools