import { ClickHouseClient, ResultSet } from "@clickhouse/client";
import {
  Client as TemporalClient,
  Connection,
  ConnectionOptions,
} from "@temporalio/client";
import { StringValue } from "@temporalio/common";
import { randomUUID } from "node:crypto";
import * as path from "path";
import * as fs from "fs";
import { Column } from "../dataModels/dataModelTypes";
import { AggregationFunction } from "../dataModels/typeConvert";

/**
 * Convert the JS type (source is JSON format by API query parameter) to the corresponding ClickHouse type for generating named placeholder of parameterized query.
 * Only support to convert number to Int or Float, boolean to Bool, string to String, other types will convert to String.
 * If exist complex type e.g: object, Array, null, undefined, Date, Record.. etc, just convert to string type by ClickHouse function in SQL.
 * ClickHouse support converting string to other types function.
 * Please see Each section of the https://clickhouse.com/docs/en/sql-reference/functions and https://clickhouse.com/docs/en/sql-reference/functions/type-conversion-functions
 * @param value
 * @returns 'Float', 'Int', 'Bool', 'String'
 */
export const mapToClickHouseType = (value: Value) => {
  if (typeof value === "number") {
    // infer the float or int according to exist remainder or not
    if (value % 1 !== 0) return "Float";
    return "Int";
  }
  // When define column type or query result with parameterized query, The Bool or Boolean type both supported.
  // But the column type of query result only return Bool, so we only support Bool type for safety.
  if (typeof value === "boolean") return "Bool";
  if (value instanceof Date) return "DateTime";
  if (Array.isArray(value)) {
    const [type, _] = value;
    return type;
  }
  return "String";
};

export const getValueFromParameter = (value: any) => {
  if (Array.isArray(value)) {
    const [type, val] = value;
    if (type === "Identifier") return val;
  }
  return value;
};

export function createClickhouseParameter(
  parameterIndex: number,
  value: Value,
) {
  // ClickHouse use {name:type} be a placeholder, so if we only use number string as name e.g: {1:Unit8}
  // it will face issue when converting to the query params => {1: value1}, because the key is value not string type, so here add prefix "p" to avoid this issue.
  return `{p${parameterIndex}:${mapToClickHouseType(value)}}`;
}

// source https://github.com/blakeembrey/sql-template-tag/blob/main/src/index.ts
/**
 * Values supported by SQL engine.
 */
export type Value = string | number | boolean | Date | [string, string];

/**
 * Supported value or SQL instance.
 */
export type RawValue = Value | Sql;

const isColumn = (value: RawValue | Column): value is Column =>
  typeof value === "object" && "name" in value;

/**
 * A SQL instance can be nested within each other to build SQL strings.
 */
export class Sql {
  readonly values: Value[];
  readonly strings: string[];

  constructor(
    rawStrings: readonly string[],
    rawValues: readonly (RawValue | Column)[],
  ) {
    if (rawStrings.length - 1 !== rawValues.length) {
      if (rawStrings.length === 0) {
        throw new TypeError("Expected at least 1 string");
      }

      throw new TypeError(
        `Expected ${rawStrings.length} strings to have ${
          rawStrings.length - 1
        } values`,
      );
    }

    const valuesLength = rawValues.reduce<number>(
      (len: number, value: RawValue | Column) =>
        len +
        (value instanceof Sql ? value.values.length : isColumn(value) ? 0 : 1),
      0,
    );

    this.values = new Array(valuesLength);
    this.strings = new Array(valuesLength + 1);

    this.strings[0] = rawStrings[0];

    // Iterate over raw values, strings, and children. The value is always
    // positioned between two strings, e.g. `index + 1`.
    let i = 0,
      pos = 0;
    while (i < rawValues.length) {
      const child = rawValues[i++];
      const rawString = rawStrings[i];

      // Check for nested `sql` queries.
      if (child instanceof Sql) {
        // Append child prefix text to current string.
        this.strings[pos] += child.strings[0];

        let childIndex = 0;
        while (childIndex < child.values.length) {
          this.values[pos++] = child.values[childIndex++];
          this.strings[pos] = child.strings[childIndex];
        }

        // Append raw string to current string.
        this.strings[pos] += rawString;
      } else if (isColumn(child)) {
        const aggregationFunction = child.annotations.find(
          ([k, _]) => k === "aggregationFunction",
        );
        if (aggregationFunction !== undefined) {
          this.strings[pos] +=
            `${(aggregationFunction[1] as AggregationFunction).functionName}Merge(\`${child.name}\`)`;
        } else {
          this.strings[pos] += `\`${child.name}\``;
        }
        this.strings[pos] += rawString;
      } else {
        this.values[pos++] = child;
        this.strings[pos] = rawString;
      }
    }
  }
}

export function sql(
  strings: readonly string[],
  ...values: readonly (RawValue | Column)[]
) {
  return new Sql(strings, values);
}

function emptyIfUndefined(value: string | undefined): string {
  return value === undefined ? "" : value;
}

export class MooseClient {
  query: QueryClient;
  workflow: WorkflowClient;

  constructor(queryClient: QueryClient, temporalClient?: TemporalClient) {
    this.query = queryClient;
    this.workflow = new WorkflowClient(temporalClient);
  }
}

export class QueryClient {
  client: ClickHouseClient;
  query_id_prefix: string;
  constructor(client: ClickHouseClient, query_id_prefix: string) {
    this.client = client;
    this.query_id_prefix = query_id_prefix;
  }

  async execute<T = any>(
    sql: Sql,
  ): Promise<ResultSet<"JSONEachRow"> & { __query_result_t?: T[] }> {
    const parameterizedStubs = sql.values.map((v, i) =>
      createClickhouseParameter(i, v),
    );

    const query = sql.strings
      .map((s, i) =>
        s != "" ? `${s}${emptyIfUndefined(parameterizedStubs[i])}` : "",
      )
      .join("");

    const query_params = sql.values.reduce(
      (acc: Record<string, unknown>, v, i) => ({
        ...acc,
        [`p${i}`]: getValueFromParameter(v),
      }),
      {},
    );

    return this.client.query({
      query,
      query_params,
      format: "JSONEachRow",
      query_id: this.query_id_prefix + randomUUID(),
    });
  }
}

interface WorkflowConfig {
  name: string;
  schedule: string;
  retries: number;
  timeout: string;
  tasks: string[];
}

export class WorkflowClient {
  client: TemporalClient | undefined;

  constructor(temporalClient?: TemporalClient) {
    this.client = temporalClient;
  }

  async execute(name: string, input_data: any) {
    try {
      if (!this.client) {
        return {
          status: 404,
          body: `Temporal client not found. Is the feature flag enabled?`,
        };
      }

      const configs = await this.loadConsolidatedConfigs();
      const config = configs[name];
      if (!config) {
        return {
          status: 404,
          body: `Workflow config not found for ${name}`,
        };
      }

      const runId = await this.startWorkflowAsync(name, config, input_data);

      return {
        status: 200,
        body: `Workflow started: ${name}. View it in the Temporal dashboard: http://localhost:8080/namespaces/default/workflows/${name}/${runId}/history`,
      };
    } catch (error) {
      return {
        status: 400,
        body: `Error starting workflow: ${error}`,
      };
    }
  }

  private async startWorkflowAsync(
    name: string,
    config: WorkflowConfig,
    input_data: any,
  ) {
    console.log(
      `API starting workflow ${name} with config ${JSON.stringify(config)} and input_data ${JSON.stringify(input_data)}`,
    );

    const handle = await this.client!.workflow.start("ScriptWorkflow", {
      args: [`${process.cwd()}/app/scripts/${name}`, input_data],
      taskQueue: "typescript-script-queue",
      workflowId: name,
      workflowIdConflictPolicy: "FAIL",
      workflowIdReusePolicy: "ALLOW_DUPLICATE",
      retry: {
        maximumAttempts: config.retries,
      },
      workflowRunTimeout: config.timeout as StringValue,
    });

    return handle.firstExecutionRunId;
  }

  private async loadConsolidatedConfigs(): Promise<
    Record<string, WorkflowConfig>
  > {
    const configPath = path.join(
      process.cwd(),
      ".moose",
      "workflow_configs.json",
    );
    const configContent = await fs.readFileSync(configPath, "utf8");
    const configArray = JSON.parse(configContent) as WorkflowConfig[];

    const configMap = configArray.reduce(
      (map: Record<string, WorkflowConfig>, config: WorkflowConfig) => {
        if (config.name) {
          map[config.name] = config;
        }
        return map;
      },
      {},
    );

    return configMap;
  }
}

/**
 * This looks similar to the client in runner.ts which is a worker.
 * Temporal SDK uses similar looking connection options & client,
 * but there are different libraries for a worker & client like this one
 * that triggers workflows.
 */
export async function getTemporalClient(
  temporalUrl: string,
  clientCert: string,
  clientKey: string,
  apiKey: string,
): Promise<TemporalClient | undefined> {
  try {
    let namespace = "default";
    if (!temporalUrl.includes("localhost")) {
      // Remove port and just get <namespace>.<account>
      const hostPart = temporalUrl.split(":")[0];
      const match = hostPart.match(/^([^.]+\.[^.]+)/);
      if (match && match[1]) {
        namespace = match[1];
      }
    }
    console.info(`Using namespace from URL: ${namespace}`);

    let connectionOptions: ConnectionOptions = {
      address: temporalUrl,
      connectTimeout: "3s",
    };

    if (!temporalUrl.includes("localhost")) {
      // URL with mTLS uses gRPC namespace endpoint which is what temporalUrl already is
      if (clientCert && clientKey) {
        console.log("Using TLS for non-local Temporal");
        const cert = await fs.readFileSync(clientCert);
        const key = await fs.readFileSync(clientKey);

        connectionOptions.tls = {
          clientCertPair: { crt: cert, key: key },
        };
      } else if (apiKey) {
        console.log("Using API key for non-local Temporal");
        // URL with API key uses gRPC regional endpoint
        connectionOptions.address = "us-west1.gcp.api.temporal.io:7233";
        connectionOptions.apiKey = apiKey;
        connectionOptions.tls = {};
        connectionOptions.metadata = {
          "temporal-namespace": namespace,
        };
      }
    }

    console.log(`<api> Connecting to Temporal at ${connectionOptions.address}`);
    const connection = await Connection.connect(connectionOptions);
    const client = new TemporalClient({ connection, namespace });
    console.log("<api> Connected to Temporal server");

    return client;
  } catch (error) {
    console.warn(`Failed to connect to Temporal. Is the feature flag enabled?`);
    return undefined;
  }
}

export const ConsumptionHelpers = {
  column: (value: string) => ["Identifier", value] as [string, string],
  table: (value: string) => ["Identifier", value] as [string, string],
};

export function join_queries({
  values,
  separator = ",",
  prefix = "",
  suffix = "",
}: {
  values: readonly RawValue[];
  separator?: string;
  prefix?: string;
  suffix?: string;
}) {
  if (values.length === 0) {
    throw new TypeError(
      "Expected `join([])` to be called with an array of multiple elements, but got an empty array",
    );
  }

  return new Sql(
    [prefix, ...Array(values.length - 1).fill(separator), suffix],
    values,
  );
}
