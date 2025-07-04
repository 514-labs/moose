# Cursor Background Agent Analysis & Setup

## Repository Analysis

This is a **polyglot monorepo** for the Moose framework - a data engineering platform. The repository contains:

### Architecture Overview
- **Language Mix**: Rust (core), TypeScript (packages), Python (bindings)
- **Build System**: Cargo workspaces + pnpm workspaces
- **Package Management**: Rust crates, npm packages, Python packages
- **Protocol Buffers**: Shared definitions between languages

### Key Components

#### 1. Core Rust Applications
- **`apps/framework-cli/`**: Main CLI tool (Cargo package)
  - Entry point: `moose-cli` binary
  - Dependencies: 60+ crates including `openssl`, `tokio`, `rdkafka`, `clickhouse`
  - Protocol buffer code generation via `build.rs`
  - Python bindings via `maturin`

#### 2. TypeScript Packages
- **`packages/ts-moose-lib/`**: Core TypeScript library
- **`packages/ts-connector-api/`**: API connectors
- **`packages/ts-connector-s3/`**: S3 connectors
- **`packages/design-system-*`**: UI components
- **`apps/create-moose-app/`**: Project scaffolding tool

#### 3. Python Components
- **`packages/py-moose-lib/`**: Python library with utilities
- **CLI Python bindings**: Built with `maturin` from Rust code

#### 4. Protocol Buffers
- **`packages/protobuf/infrastructure_map.proto`**: Core schema definitions
- Compiled to Rust, TypeScript, and Python bindings

### Build Dependencies Identified

From analyzing the codebase, CI workflows, and project files:

1. **System Requirements**:
   - Ubuntu 22.04+ (or equivalent)
   - Build tools: `gcc`, `make`, `pkg-config`
   - SSL/TLS libraries: `libssl-dev`
   - Database libs: `libsqlite3-dev`, `libpq-dev`

2. **Language Runtimes**:
   - **Rust**: Latest stable (for CLI and core libraries)
   - **Node.js**: 20.x LTS (from project requirements)
   - **Python**: 3.12+ (from `pyproject.toml`)
   - **pnpm**: 9.9.0 (from `package.json`)

3. **Build Tools**:
   - **protoc**: 24.4 (from CI workflow)
   - **maturin**: 1.8.1 (for Python-Rust bindings)

## Cursor Background Agent Setup

### 1. Environment Configuration (`.cursor/environment.json`)

Created a comprehensive Docker-based environment with:

```json
{
  "dockerfile": "FROM ubuntu:22.04...",
  "install": "pnpm install --frozen-lockfile",
  "start": "echo 'Environment setup complete. Ready for development!'",
  "terminals": []
}
```

### 2. Dockerfile Features

The Docker environment includes:

- **Base**: Ubuntu 22.04 LTS
- **System packages**: All required build dependencies
- **Node.js 20.x**: Installed via NodeSource repository
- **pnpm 9.9.0**: Exact version matching project requirements
- **Rust toolchain**: Latest stable via rustup
- **Python 3.12**: Upgraded from Ubuntu's default 3.10
- **Protocol Buffers**: Version 24.4 matching CI pipeline
- **Maturin**: For Python-Rust bindings

### 3. Installation Strategy

- **System-level**: All dependencies installed in Docker image
- **Project-level**: `pnpm install --frozen-lockfile` during setup
- **Verification**: All tools verified during image build

### 4. Verification Tools

Created `scripts/verify-environment.sh` to check:
- All required commands are available
- Correct versions are installed
- Project structure is intact
- Build dependencies are satisfied

## Usage Instructions

### For Background Agent Users

1. **Start Agent**: Use `Ctrl+E` in Cursor to launch Background Agent
2. **Select Repository**: Choose this repository
3. **Auto-Setup**: Environment builds automatically from `.cursor/environment.json`
4. **Verify**: Run `./scripts/verify-environment.sh` to confirm setup

### Development Workflow

```bash
# Verify environment
./scripts/verify-environment.sh

# Install dependencies (automatic during setup)
pnpm install --frozen-lockfile

# Build Rust CLI
cd apps/framework-cli
cargo build

# Build TypeScript packages
pnpm build

# Build specific packages
pnpm --filter=@514labs/moose-lib run build

# Run tests
cargo test
pnpm test
```

### Common Commands

- **CLI Development**: `cargo run --bin moose-cli`
- **TypeScript Watch**: `pnpm --filter=@514labs/moose-lib run dev`
- **Python Development**: `cd packages/py-moose-lib && python3 -m pytest`
- **Protocol Buffers**: Automatically compiled via `build.rs`

## Key Benefits

### 1. Complete Environment
- All dependencies pre-installed
- No manual setup required
- Consistent across all agents

### 2. Multi-Language Support
- Rust, TypeScript, Python all configured
- Cross-language builds work seamlessly
- Protocol buffer compilation included

### 3. Matches CI/CD
- Same dependency versions as GitHub Actions
- Identical build environment
- Predictable build results

### 4. Development Ready
- All tools available immediately
- Fast iteration cycles
- Full debugging capabilities

## Troubleshooting

### Common Issues

1. **Environment Reset**: Delete agent and recreate
2. **Version Mismatches**: Check `verify-environment.sh` output
3. **Build Failures**: Ensure all dependencies installed
4. **Python Issues**: Verify Python 3.12 is default

### Support Resources

- **Documentation**: `CURSOR_BACKGROUND_AGENT_SETUP.md`
- **Verification**: `scripts/verify-environment.sh`
- **Cursor Support**: Discord #background-agent channel

## Summary

The Cursor Background Agent setup provides a complete, production-ready development environment for the Moose framework. It handles the complexity of the polyglot build system while providing a seamless development experience.

**Key Achievements**:
- ✅ All required dependencies installed
- ✅ Multi-language support (Rust, TypeScript, Python)
- ✅ Protocol buffer compilation
- ✅ Matches CI/CD environment
- ✅ Comprehensive verification tools
- ✅ Ready for immediate development