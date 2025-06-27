import { ClickHouseClient, ResultSet } from "@clickhouse/client";
import {
  Client as TemporalClient,
  Connection,
  ConnectionOptions,
} from "@temporalio/client";
import { StringValue } from "@temporalio/common";
import { createHash, randomUUID } from "node:crypto";
import * as path from "path";
import * as fs from "fs";
import { Column } from "../dataModels/dataModelTypes";
import { OlapTable } from "../dmv2";
import { getWorkflows } from "../dmv2/internal";
import { JWTPayload } from "jose";
import { Sql, Value, RawValue, toQuery } from "../sqlHelpers";

export interface ConsumptionUtil {
  client: MooseClient;

  // SQL interpolator
  sql: typeof sql;
  jwt: JWTPayload | undefined;
}

export function sql(
  strings: readonly string[],
  ...values: readonly (RawValue | Column | OlapTable<any>)[]
) {
  return new Sql(strings, values);
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
    const [query, query_params] = toQuery(sql);

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

      // Get workflow configuration (DMv2 or legacy)
      const config = await this.getWorkflowConfig(name);

      // Process input data and generate workflow ID
      const [processedInput, workflowId] = this.processInputData(
        name,
        input_data,
      );

      console.log(
        `WorkflowClient - starting ${config.is_dmv2 ? "DMv2 " : ""}workflow: ${name} with config ${JSON.stringify(config)} and input_data ${JSON.stringify(processedInput)}`,
      );

      // Start workflow with appropriate args
      const workflowArgs = this.buildWorkflowArgs(
        name,
        processedInput,
        config.is_dmv2,
      );

      const handle = await this.client.workflow.start("ScriptWorkflow", {
        args: workflowArgs,
        taskQueue: "typescript-script-queue",
        workflowId,
        workflowIdConflictPolicy: "FAIL",
        workflowIdReusePolicy: "ALLOW_DUPLICATE",
        retry: {
          maximumAttempts: config.retries,
        },
        workflowRunTimeout: config.timeout as StringValue,
      });

      return {
        status: 200,
        body: `Workflow started: ${name}. View it in the Temporal dashboard: http://localhost:8080/namespaces/default/workflows/${workflowId}/${handle.firstExecutionRunId}/history`,
      };
    } catch (error) {
      return {
        status: 400,
        body: `Error starting workflow: ${error}`,
      };
    }
  }

  private async getWorkflowConfig(
    name: string,
  ): Promise<{ retries: number; timeout: string; is_dmv2: boolean }> {
    // Check for DMv2 workflow first
    try {
      const workflows = await getWorkflows();
      const dmv2Workflow = workflows.get(name);
      if (dmv2Workflow) {
        return {
          retries: dmv2Workflow.config.retries || 3,
          timeout: dmv2Workflow.config.timeout || "1h",
          is_dmv2: true,
        };
      }
    } catch (error) {
      // Fall through to legacy config
    }

    // Fall back to legacy configuration
    const configs = await this.loadConsolidatedConfigs();
    const config = configs[name];
    if (!config) {
      throw new Error(`Workflow config not found for ${name}`);
    }

    return {
      retries: config.retries || 3,
      timeout: config.timeout || "1h",
      is_dmv2: false,
    };
  }

  private processInputData(name: string, input_data: any): [any, string] {
    let workflowId = name;
    if (input_data) {
      const hash = createHash("sha256")
        .update(JSON.stringify(input_data))
        .digest("hex")
        .slice(0, 16);
      workflowId = `${name}-${hash}`;
    }
    return [input_data, workflowId];
  }

  private buildWorkflowArgs(
    name: string,
    input_data: any,
    is_dmv2: boolean,
  ): any[] {
    if (is_dmv2) {
      return [name, input_data];
    } else {
      return [`${process.cwd()}/app/scripts/${name}`, input_data];
    }
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
    console.info(`<api> Using namespace from URL: ${namespace}`);

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
