<a href="https://docs.fiveonefour.com/moose/"><img src="https://raw.githubusercontent.com/514-labs/moose/main/logo-m-light.png" alt="moose logo" height="100px"></a>

[![NPM Version](https://img.shields.io/npm/v/%40514labs%2Fmoose-cli?logo=npm)](https://www.npmjs.com/package/@514labs/moose-cli?activeTab=readme)
[![Moose Community](https://img.shields.io/badge/slack-moose_community-purple.svg?logo=slack)](https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg)
[![Docs](https://img.shields.io/badge/quick_start-docs-blue.svg)](https://docs.fiveonefour.com/moose/getting-started/quickstart)
[![MIT license](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)

# MooseStack

**Type‑safe, code‑first framework for real‑time analytics** — MooseStack ties together an OLAP database (ClickHouse), streaming (Redpanda), workflow orchestration (Temporal), and state management (Redis) into a single toolkit for building analytical backends.

Model your data, streams, and APIs in TypeScript or Python and deploy in minutes. Power OLAP databases, high‑throughput ingestion, ETL workflows, and query APIs from one modular codebase.

## Why MooseStack?

- **5‑minute setup**: Install the CLI and [bootstrap a backend](https://docs.fiveonefour.com/moose/getting-started/quickstart).
- **Pre‑integrated components**: ClickHouse (storage), Redpanda (streaming), Temporal (orchestration).
- **Hot‑reload development**: Run everything locally with live schema migrations.
- **Code‑first infrastructure**: Declare tables, streams, workflows, and APIs in TS/Python; the framework wires it up.
- **Modular design**: Only enable the modules you need. Each module is independent and can be adopted incrementally.
- **Built for speed**: ClickHouse is a columnar database that is roughly 100× faster than traditional databases for analytical queries.

## Modules

- **OLAP**: Manage ClickHouse tables, materialized views, and migrations in code.
- **Streaming**: Real‑time pipelines with Kafka/Redpanda and transformation functions.
- **Workflows**: ETL pipelines and tasks with Temporal.
- **APIs**: Type‑safe ingestion and query endpoints with auto‑generated OpenAPI docs.
- **Tooling**: `moose build`, `moose plan`, and built‑in observability endpoints.

## Quickstart

### Install the CLI

```bash
bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose
```

### Create a project

```bash
moose init my-project --from-remote <YOUR_CLICKHOUSE_CONNECTION_STRING> --language typescript
# or
moose init my-project --from-remote <YOUR_CLICKHOUSE_CONNECTION_STRING> --language python
```

### Run locally

```bash
cd my-project
moose dev
```

Moose will start ClickHouse, Redpanda, Temporal, and Redis; the CLI validates each component.

### Example

TypeScript: define a ClickHouse table, a Redpanda stream, and ingestion/consumption APIs.

```typescript
import { Key, OlapTable, Stream, IngestApi, ConsumptionApi } from "@514labs/moose-lib";

interface Event { id: Key<string>; name: string; createdAt: Date; }

const events = new OlapTable<Event>("events");
const eventsStream = new Stream<Event>("events-stream", { destination: events });

new IngestApi<Event>("post-event", { destination: eventsStream });
new ConsumptionApi<{ limit?: number }, Event[]>("get-events", async ({ limit = 10 }, { client, sql }) => {
  const result = await client.query.execute(sql`SELECT * FROM ${events} LIMIT ${limit}`);
  return await result.json();
});
```

## Built on

- ClickHouse (OLAP storage)
- Redpanda (streaming)
- Temporal (workflow orchestration)
- Redis (internal state)

## Docs

Start with the [Quickstart](https://docs.fiveonefour.com/moose/getting-started/quickstart) and explore deployment, operations, and APIs.

## Moose in Production

Moose is beta software and under active development. Multiple public companies are using Moose in production. If you’re evaluating production use, consider [Boreal Cloud](https://www.fiveonefour.com/boreal) or review the documentation for [self-hosting](https://docs.fiveonefour.com/moose/deploying). You can also reach us at [moose@fiveonefour.com](mailto:moose@fiveonefour.com) or join the [slack community](https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg).

## Community

Join us on Slack: https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg

## Contributing

We welcome contributions! See the [contribution guidelines](https://github.com/514-labs/moose/blob/main/CONTRIBUTING.md).

## License

MooseStack is open source software and MIT licensed.
