# PostgreSQL Port Hardcoding Issue and Fix

## Issue Description

When users configure a workflow's PostgreSQL database port to 5433 (instead of the default 5432) in their `moose.config.toml`, they encounter a communication error. The system was incorrectly configuring inter-container communication to use the host port instead of the internal Docker network port.

## Root Cause

The issue was in the `apps/framework-cli/src/utilities/docker-compose.yml.hbs` template file where the temporal container was configured to connect to PostgreSQL using the host-mapped port:

```yaml
environment:
  - DB_PORT=${TEMPORAL_DB_PORT:-5432}
```

This caused the temporal container to try to connect to PostgreSQL on port 5433, but within the Docker network, PostgreSQL always runs on its default internal port 5432.

## Docker Network Communication

In Docker Compose:
- **Host port mapping**: `- ${TEMPORAL_DB_PORT:-5432}:5432` maps host port 5433 to container port 5432
- **Inter-container communication**: Containers communicate using internal ports (5432), not host-mapped ports (5433)

## System Flow

1. User configures `db_port = 5433` in `moose.config.toml`
2. Configuration is loaded into `TemporalConfig` struct with `db_port = 5433`
3. `TemporalConfig.to_env_vars()` converts this to `TEMPORAL_DB_PORT=5433`
4. PostgreSQL container exposes port 5433 on host but runs on port 5432 internally
5. Temporal container incorrectly tries to connect to PostgreSQL on port 5433 (fails)
6. **Fix**: Temporal container should always connect to PostgreSQL on port 5432 (internal port)

## Solution

Fixed the docker-compose template to use the correct internal port for inter-container communication:

**Before (broken):**
```yaml
environment:
  - DB_PORT=${TEMPORAL_DB_PORT:-5432}
```

**After (fixed):**
```yaml
environment:
  - DB_PORT=5432
```

The temporal container now always connects to PostgreSQL on port 5432 (the internal Docker network port), while PostgreSQL can still be exposed on any host port (like 5433) for external access.

## Files Modified

1. `apps/framework-cli/src/utilities/docker-compose.yml.hbs` - Fixed DB_PORT environment variable

## Testing

To test the fix:

1. Set `db_port = 5433` in your `moose.config.toml`
2. Run `moose dev` to start the development environment
3. Verify that PostgreSQL is accessible on host port 5433: `psql -h localhost -p 5433 -U temporal`
4. Run workflow commands to confirm inter-container communication works correctly

## Configuration Example

```toml
[temporal_config]
db_user = "temporal"
db_password = "temporal"
db_port = 5433  # Custom host port - PostgreSQL will be accessible on localhost:5433
temporal_host = "localhost"
temporal_port = 7233
```

This will now correctly:
- Expose PostgreSQL on host port 5433 for external access
- Use internal port 5432 for temporal-to-postgresql communication within Docker network