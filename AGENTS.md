
# Moose Development Guide

## Build, Lint, and Test

- **Build:** `pnpm build` (JS/TS), `cargo build` (Rust)
- **Lint:** `pnpm lint` (JS/TS), `cargo clippy` (Rust - always run before commits)
- **Format:** `pnpm format` (JS/TS), `rustfmt --edition 2021` (Rust)
- **Test:** `pnpm test` (JS/TS), `cargo test` (Rust), `pytest apps/framework-cli/tests/test_basic.py` (Python)
- **Single test:** `mocha -r ts-node/register 'src/**/*.test.ts'` (TS), `cargo test test_name` (Rust)

## Code Style

- **Formatting:** Prettier (default settings) for JS/TS, rustfmt for Rust
- **Linting:** ESLint (custom config) for JS/TS, Clippy for Rust
- **Imports:** Keep grouped and sorted
- **Types:** Use TypeScript types and Python type hints
- **Naming:** camelCase (TS), snake_case (Python), snake_case (Rust)
- **Error Handling:** Use try/catch (JS/TS), thiserror crate (Rust) - NO anyhow::Result
- **Constants:** Use `const` in Rust, place in `constants.rs` at appropriate module level
- **Git:** Follow conventional commit messages

## Rust-Specific Rules

- Use thiserror for error definitions, avoid anyhow::Result
- Define error types near their unit of fallibility (no global Error types)
- Use newtypes as tuple structs with validation constructors
- Run `cargo clippy` before all commits - trust Clippy's guidance
- Document all public APIs and breaking changes

## Project Structure

PNPM monorepo with Turbo (JS/TS) + Cargo (Rust):
- `apps/`: CLI, docs, e2e tests, distribution packages
- `packages/`: Shared libraries (TS/Python moose libs, connectors, design system)
- `templates/`: Standalone Moose project templates
