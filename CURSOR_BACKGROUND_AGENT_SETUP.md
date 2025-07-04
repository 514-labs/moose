# Cursor Background Agent Setup Guide

This repository is configured for use with Cursor Background Agents, which allows you to spawn asynchronous agents that can edit and run code in a remote environment.

## What's Included

The `.cursor/environment.json` file sets up a complete development environment with:

### System Dependencies
- **Ubuntu 22.04** base image
- Essential build tools (gcc, make, pkg-config, etc.)
- SSL/TLS libraries for secure connections
- Database development libraries (SQLite, PostgreSQL)
- Archive utilities (zip, unzip)

### Language Runtime Requirements

#### Rust Toolchain
- **Rust stable** with Cargo package manager
- Required for building the main `framework-cli` application
- Includes `maturin` for Python-Rust bindings

#### Node.js Environment
- **Node.js 20.x** (LTS version)
- **pnpm 9.9.0** for workspace management
- Required for TypeScript packages and frontend components

#### Python Environment
- **Python 3.12** (upgraded from Ubuntu's default 3.10)
- **pip** package manager
- **venv** for virtual environment support
- Required for Python packages and CLI Python bindings

#### Protocol Buffers
- **protoc 24.4** compiler
- Required for building `.proto` definitions in `packages/protobuf/`
- Matches the version used in CI/CD pipeline

## How to Use

### 1. Start a Background Agent

In Cursor, press `Ctrl+E` to open the Background Agent control panel, then:
1. Click "New Agent"
2. Select this repository
3. The agent will automatically set up the environment using the configuration

### 2. Building the Project

Once the agent is running, you can build the entire project:

```bash
# Install all dependencies (automatically run during setup)
pnpm install --frozen-lockfile

# Build the Rust CLI
cd apps/framework-cli
cargo build

# Build TypeScript packages
pnpm build

# Build specific packages
pnpm --filter=@514labs/moose-lib run build
```

### 3. Development Workflow

The environment supports all development workflows:

```bash
# Run tests
cargo test  # Rust tests
pnpm test   # TypeScript tests

# Development servers
cd apps/framework-cli
cargo run -- dev  # Run the CLI in development mode

# Python development
cd packages/py-moose-lib
python3 -m pytest  # Run Python tests
```

### 4. Available Commands

The agent environment includes all necessary tools:

- `cargo` - Rust package manager and build tool
- `pnpm` - Node.js package manager (workspace-aware)
- `python3` - Python 3.12 interpreter
- `pip` - Python package installer
- `protoc` - Protocol buffer compiler
- `maturin` - Python-Rust binding builder

## Project Structure

This is a polyglot monorepo with:

```
moose/
├── apps/
│   ├── framework-cli/          # Main Rust CLI application
│   ├── create-moose-app/       # TypeScript app generator
│   ├── framework-docs/         # Documentation site
│   └── framework-internal-app/ # Internal application
├── packages/
│   ├── py-moose-lib/          # Python library
│   ├── ts-moose-lib/          # TypeScript library
│   ├── posthog514client-rs/   # Rust analytics client
│   ├── protobuf/              # Protocol buffer definitions
│   └── [other packages]/
├── templates/                  # Project templates
└── examples/                   # Example projects
```

## Key Features

### Rust Components
- **framework-cli**: The main CLI tool built with Rust
- **posthog514client-rs**: Analytics client library
- Uses `protobuf-codegen` for generating Rust code from `.proto` files

### TypeScript Components
- **moose-lib**: Core TypeScript library
- **connectors**: Data source connectors (API, S3, etc.)
- **design-system**: UI components and utilities

### Python Components
- **py-moose-lib**: Python bindings and utilities
- **CLI Python bindings**: Maturin-built Python module

### Protocol Buffers
- Infrastructure mapping definitions
- Shared between Rust and other language bindings

## Troubleshooting

### Common Issues

1. **Build failures**: Ensure all dependencies are installed
   ```bash
   pnpm install --frozen-lockfile
   ```

2. **Protobuf compilation errors**: Verify `protoc` is installed and accessible
   ```bash
   protoc --version  # Should show version 24.4
   ```

3. **Python version issues**: Ensure Python 3.12 is the default
   ```bash
   python3 --version  # Should show Python 3.12.x
   ```

4. **Rust build issues**: Check that Rust toolchain is properly set up
   ```bash
   rustc --version
   cargo --version
   ```

### Environment Reset

If you need to reset the environment:

1. Stop the current Background Agent
2. Create a new agent - it will rebuild the environment from scratch
3. All dependencies will be reinstalled automatically

## Security Notes

- The Background Agent runs in an isolated environment
- All builds happen in the remote environment
- Code is automatically synchronized with your GitHub repository
- The agent has internet access for downloading dependencies

## Support

For issues specific to this repository setup:
- Check the build logs in the Background Agent interface
- Verify all dependencies are correctly installed
- Ensure the environment matches the specification in `.cursor/environment.json`

For Cursor Background Agent general support:
- Discord: #background-agent channel
- Email: background-agent-feedback@cursor.com