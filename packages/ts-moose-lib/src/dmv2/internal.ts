/**
 * @module internal
 * Internal implementation details for the Moose v2 data model (dmv2).
 *
 * This module manages the registration of user-defined dmv2 resources (Tables, Streams, APIs, etc.)
 * and provides functions to serialize these resources into a JSON format (`InfrastructureMap`)
 * expected by the Moose infrastructure management system. It also includes helper functions
 * to retrieve registered handler functions (for streams and egress APIs) and the base class
 * (`TypedBase`) used by dmv2 resource classes.
 *
 * @internal This module is intended for internal use by the Moose library and compiler plugin.
 *           Its API might change without notice.
 */
import process from "process";
import {
  IngestApi,
  ConsumptionApi,
  SqlResource,
  Workflow,
  Task,
} from "./index";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { Column } from "../dataModels/dataModelTypes";
import { ConsumptionUtil } from "../index";
import { OlapTable } from "./sdk/olapTable";
import { ConsumerConfig, Stream, TransformConfig } from "./sdk/stream";
import { compilerLog } from "../commons";

/**
 * Internal registry holding all defined Moose dmv2 resources.
 * Populated by the constructors of OlapTable, Stream, IngestApi, etc.
 * Accessed via `getMooseInternal()`.
 */
const moose_internal = {
  tables: new Map<string, OlapTable<any>>(),
  streams: new Map<string, Stream<any>>(),
  ingestApis: new Map<string, IngestApi<any>>(),
  egressApis: new Map<string, ConsumptionApi<any>>(),
  sqlResources: new Map<string, SqlResource>(),
  workflows: new Map<string, Workflow>(),
};
/**
 * Default retention period for streams if not specified (7 days in seconds).
 */
const defaultRetentionPeriod = 60 * 60 * 24 * 7;

/**
 * JSON representation of an OLAP table configuration.
 */
interface TableJson {
  /** The name of the table. */
  name: string;
  /** Array defining the table's columns and their types. */
  columns: Column[];
  /** List of column names used for the ORDER BY clause. */
  orderBy: string[];
  /** Flag indicating if the table uses a deduplicating engine (e.g., ReplacingMergeTree). */
  deduplicate: boolean;
  /** The name of the ClickHouse engine (e.g., "MergeTree", "ReplacingMergeTree"). */
  engine?: string;
  /** Optional version string for the table configuration. */
  version?: string;
  /** Optional metadata for the table (e.g., description). */
  metadata?: { description?: string };
  /** Lifecycle management setting for the table. */
  lifeCycle?: string;
}
/**
 * Represents a target destination for data flow, typically a stream.
 */
interface Target {
  /** The name of the target resource (e.g., stream name). */
  name: string;
  /** The kind of the target resource. */
  kind: "stream"; // may add `| "table"` in the future
  /** Optional version string of the target resource's configuration. */
  version?: string;
  /** Optional metadata for the target (e.g., description for function processes). */
  metadata?: { description?: string };
}

/**
 * Represents a consumer attached to a stream.
 */
interface Consumer {
  /** Optional version string for the consumer configuration. */
  version?: string;
}

/**
 * JSON representation of a Stream/Topic configuration.
 */
interface StreamJson {
  /** The name of the stream/topic. */
  name: string;
  /** Array defining the message schema (columns/fields). */
  columns: Column[];
  /** Data retention period in seconds. */
  retentionPeriod: number;
  /** Number of partitions for the stream/topic. */
  partitionCount: number;
  /** Optional name of the OLAP table this stream automatically syncs to. */
  targetTable?: string;
  /** Optional version of the target OLAP table configuration. */
  targetTableVersion?: string;
  /** Optional version string for the stream configuration. */
  version?: string;
  /** List of target streams this stream transforms data into. */
  transformationTargets: Target[];
  /** Flag indicating if a multi-transform function (`_multipleTransformations`) is defined. */
  hasMultiTransform: boolean;
  /** List of consumers attached to this stream. */
  consumers: Consumer[];
  /** Optional description for the stream. */
  metadata?: { description?: string };
  /** Lifecycle management setting for the stream. */
  lifeCycle?: string;
}
/**
 * JSON representation of an Ingest API configuration.
 */
interface IngestApiJson {
  /** The name of the Ingest API endpoint. */
  name: string;
  /** Array defining the expected input schema (columns/fields). */
  columns: Column[];

  /** The target stream where ingested data is written. */
  writeTo: Target;
  /** The DLQ if the data does not fit the schema. */
  deadLetterQueue?: string;
  /** Optional version string for the API configuration. */
  version?: string;
  /** Optional description for the API. */
  metadata?: { description?: string };
}

/**
 * JSON representation of an Egress (Consumption) API configuration.
 */
interface EgressApiJson {
  /** The name of the Egress API endpoint. */
  name: string;
  /** Array defining the expected query parameters schema. */
  queryParams: Column[];
  /** JSON schema definition of the API's response body. */
  responseSchema: IJsonSchemaCollection.IV3_1;
  /** Optional version string for the API configuration. */
  version?: string;
  /** Optional description for the API. */
  metadata?: { description?: string };
}

/**
 * Represents the unique signature of an infrastructure component (Table, Topic, etc.).
 * Used for defining dependencies between SQL resources.
 */
interface InfrastructureSignatureJson {
  /** A unique identifier for the resource instance (often name + version). */
  id: string;
  /** The kind/type of the infrastructure component. */
  kind:
    | "Table"
    | "Topic"
    | "ApiEndpoint"
    | "TopicToTableSyncProcess"
    | "View"
    | "SqlResource";
}

interface WorkflowJson {
  name: string;
  retries?: number;
  timeout?: string;
  schedule?: string;
}

/**
 * JSON representation of a generic SQL resource (like View, MaterializedView).
 */
interface SqlResourceJson {
  /** The name of the SQL resource. */
  name: string;
  /** Array of SQL DDL statements required to create the resource. */
  setup: readonly string[];
  /** Array of SQL DDL statements required to drop the resource. */
  teardown: readonly string[];

  /** List of infrastructure components (by signature) that this resource reads from. */
  pullsDataFrom: InfrastructureSignatureJson[];
  /** List of infrastructure components (by signature) that this resource writes to. */
  pushesDataTo: InfrastructureSignatureJson[];
}

/**
 * Converts the internal resource registry into a structured infrastructure map.
 * This map is serialized to JSON and used by the Moose infrastructure system.
 *
 * @param registry The internal Moose resource registry (`moose_internal`).
 * @returns An object containing dictionaries of tables, topics, ingest APIs, egress APIs, and SQL resources, formatted according to the `*Json` interfaces.
 */
export const toInfraMap = (registry: typeof moose_internal) => {
  const tables: { [key: string]: TableJson } = {};
  const topics: { [key: string]: StreamJson } = {};
  const ingestApis: { [key: string]: IngestApiJson } = {};
  const egressApis: { [key: string]: EgressApiJson } = {};
  const sqlResources: { [key: string]: SqlResourceJson } = {};
  const workflows: { [key: string]: WorkflowJson } = {};

  registry.tables.forEach((table, key) => {
    // If the table is part of an IngestPipeline, inherit metadata if not set
    let metadata = (table as any).metadata;
    if (!metadata && table.config && (table as any).pipelineParent) {
      metadata = (table as any).pipelineParent.metadata;
    }

    const tableKey =
      table.config.version ?
        `${table.name}_${table.config.version}`
      : table.name;

    tables[tableKey] = {
      name: table.name,
      columns: table.columnArray,
      orderBy: table.config.orderByFields ?? [],
      deduplicate: table.config.deduplicate ?? false,
      engine: table.config.engine,
      version: table.config.version,
      metadata,
      lifeCycle: table.config.lifeCycle,
    };
  });

  registry.streams.forEach((stream, key) => {
    // If the stream is part of an IngestPipeline, inherit metadata if not set
    let metadata = stream.metadata;
    if (!metadata && stream.config && (stream as any).pipelineParent) {
      metadata = (stream as any).pipelineParent.metadata;
    }
    const transformationTargets: Target[] = [];
    const consumers: Consumer[] = [];

    stream._transformations.forEach((transforms, destinationName) => {
      transforms.forEach(([destination, _, config]) => {
        const destinationKey =
          config.version ?
            `${destinationName}_${config.version}`
          : destinationName;
        transformationTargets.push({
          kind: "stream",
          name: destinationKey,
          version: config.version,
          metadata: config.metadata,
        });
      });
    });

    stream._consumers.forEach((consumer) => {
      consumers.push({
        version: consumer.config.version,
      });
    });

    const streamKey =
      stream.config.version ?
        `${stream.name}_${stream.config.version}`
      : stream.name;
    topics[streamKey] = {
      name: stream.name,
      columns: stream.columnArray,
      targetTable: stream.config.destination?.name,
      targetTableVersion: stream.config.destination?.config.version,
      retentionPeriod: stream.config.retentionPeriod ?? defaultRetentionPeriod,
      partitionCount: stream.config.parallelism ?? 1,
      version: stream.config.version,
      transformationTargets,
      hasMultiTransform: stream._multipleTransformations === undefined,
      consumers,
      metadata,
      lifeCycle: stream.config.lifeCycle,
    };
  });

  registry.ingestApis.forEach((api, key) => {
    // If the ingestApi is part of an IngestPipeline, inherit metadata if not set
    let metadata = api.metadata;
    if (!metadata && api.config && (api as any).pipelineParent) {
      metadata = (api as any).pipelineParent.metadata;
    }

    let deadLetterQueueName = undefined;
    if (api.config.deadLetterQueue) {
      // Find the key that corresponds to this DLQ stream in the registry
      for (const [streamKey, stream] of registry.streams) {
        if (stream === api.config.deadLetterQueue) {
          // Convert to Rust format
          deadLetterQueueName =
            stream.config.version ?
              `${stream.name}_${stream.config.version}`
            : stream.name;
          break;
        }
      }
    }

    // Find the correct key for the destination stream in the streams registry and convert to Rust format
    let destinationStreamName = api.config.destination.name;
    for (const [streamKey, stream] of registry.streams) {
      if (stream === api.config.destination) {
        // Convert to Rust format
        destinationStreamName =
          stream.config.version ?
            `${stream.name}_${stream.config.version}`
          : stream.name;
        break;
      }
    }

    const rustKey =
      api.config.version ? `${api.name}_${api.config.version}` : api.name;
    ingestApis[rustKey] = {
      name: api.name,
      columns: api.columnArray,
      version: api.config.version,
      writeTo: {
        kind: "stream",
        name: destinationStreamName,
      },
      deadLetterQueue: deadLetterQueueName,
      metadata,
    };
  });

  registry.egressApis.forEach((api, key) => {
    const rustKey =
      api.config.version ? `${api.name}:${api.config.version}` : api.name;
    egressApis[rustKey] = {
      name: api.name,
      queryParams: api.columnArray,
      responseSchema: api.responseSchema,
      version: api.config.version,
      metadata: api.metadata,
    };
  });

  registry.sqlResources.forEach((sqlResource) => {
    sqlResources[sqlResource.name] = {
      name: sqlResource.name,
      setup: sqlResource.setup,
      teardown: sqlResource.teardown,

      pullsDataFrom: sqlResource.pullsDataFrom.map((r) => {
        if (r.kind === "OlapTable") {
          const table = r as OlapTable<any>;
          const id =
            table.config.version ?
              `${table.name}_${table.config.version}`
            : table.name;
          return {
            id,
            kind: "Table",
          };
        } else if (r.kind === "SqlResource") {
          const resource = r as SqlResource;
          return {
            id: resource.name,
            kind: "SqlResource",
          };
        } else {
          throw new Error(`Unknown sql resource dependency type: ${r}`);
        }
      }),
      pushesDataTo: sqlResource.pushesDataTo.map((r) => {
        if (r.kind === "OlapTable") {
          const table = r as OlapTable<any>;
          const id =
            table.config.version ?
              `${table.name}_${table.config.version}`
            : table.name;
          return {
            id,
            kind: "Table",
          };
        } else if (r.kind === "SqlResource") {
          const resource = r as SqlResource;
          return {
            id: resource.name,
            kind: "SqlResource",
          };
        } else {
          throw new Error(`Unknown sql resource dependency type: ${r}`);
        }
      }),
    };
  });

  registry.workflows.forEach((workflow) => {
    workflows[workflow.name] = {
      name: workflow.name,
      retries: workflow.config.retries,
      timeout: workflow.config.timeout,
      schedule: workflow.config.schedule,
    };
  });

  return {
    topics,
    tables,
    ingestApis,
    egressApis,
    sqlResources,
    workflows,
  };
};

/**
 * Retrieves the global internal Moose resource registry.
 * Uses `globalThis` to ensure a single registry instance.
 *
 * @returns The internal Moose resource registry.
 */
export const getMooseInternal = (): typeof moose_internal =>
  (globalThis as any).moose_internal;

// work around for variable visibility in compiler output
if (getMooseInternal() === undefined) {
  (globalThis as any).moose_internal = moose_internal;
}

/**
 * Loads the user's application entry point (`app/index.ts`) to register resources,
 * then generates and prints the infrastructure map as JSON.
 *
 * This function is the main entry point used by the Moose infrastructure system
 * to discover the defined resources.
 * It prints the JSON map surrounded by specific delimiters (`___MOOSE_STUFF___start`
 * and `end___MOOSE_STUFF___`) for easy extraction by the calling process.
 */
export const dumpMooseInternal = async () => {
  loadIndex();

  console.log(
    "___MOOSE_STUFF___start",
    JSON.stringify(toInfraMap(getMooseInternal())),
    "end___MOOSE_STUFF___",
  );
};

const loadIndex = () => {
  try {
    require(`${process.cwd()}/app/index.ts`);
  } catch (error) {
    let hint: string | undefined;
    const details = error instanceof Error ? error.message : String(error);
    if (details.includes("ERR_REQUIRE_ESM") || details.includes("ES Module")) {
      hint =
        "The file or its dependencies are ESM-only. Switch to packages that dual-support CJS & ESM, or upgrade to Node 22.12+. " +
        "If you must use Node 20, you may try Node 20.19";
    }

    const errorMsg = `${hint ?? ""}\n\n${details}`;
    throw new Error(errorMsg);
  }
};

/**
 * Loads the user's application entry point and extracts all registered stream
 * transformation and consumer functions.
 *
 * @returns A Map where keys are unique identifiers for transformations/consumers
 *          (e.g., "sourceStream_destStream_version", "sourceStream_<no-target>_version")
 *          and values are the corresponding handler functions.
 */
export const getStreamingFunctions = async () => {
  loadIndex();

  const registry = getMooseInternal();
  const transformFunctions = new Map<
    string,
    [(data: unknown) => unknown, TransformConfig<any> | ConsumerConfig<any>]
  >();

  registry.streams.forEach((stream) => {
    stream._transformations.forEach((transforms, destinationName) => {
      transforms.forEach(([_, transform, config]) => {
        const transformFunctionKey = `${stream.name}_${destinationName}${config.version ? `_${config.version}` : ""}`;
        compilerLog(`getStreamingFunctions: ${transformFunctionKey}`);
        transformFunctions.set(transformFunctionKey, [transform, config]);
      });
    });

    stream._consumers.forEach((consumer) => {
      const consumerFunctionKey = `${stream.name}_<no-target>${consumer.config.version ? `_${consumer.config.version}` : ""}`;
      transformFunctions.set(consumerFunctionKey, [
        consumer.consumer,
        consumer.config,
      ]);
    });
  });

  return transformFunctions;
};

/**
 * Loads the user's application entry point and extracts all registered
 * Egress API handler functions.
 *
 * @returns A Map where keys are the names of the Egress APIs and values
 *          are their corresponding handler functions.
 */
export const getEgressApis = async () => {
  loadIndex();
  const egressFunctions = new Map<
    string,
    (params: unknown, utils: ConsumptionUtil) => unknown
  >();

  const registry = getMooseInternal();
  registry.egressApis.forEach((api, key) => {
    // Use the full key (name:version) for storage
    egressFunctions.set(key, api.getHandler());

    // For backward compatibility, also store under just the name if no version is specified
    // Only set the name-only key if this API has no version (making it the default)
    const nameOnly = api.name;
    if (!api.config.version && !egressFunctions.has(nameOnly)) {
      egressFunctions.set(nameOnly, api.getHandler());
    }
  });

  return egressFunctions;
};

export const dlqSchema: IJsonSchemaCollection.IV3_1 = {
  version: "3.1",
  components: {
    schemas: {
      DeadLetterModel: {
        type: "object",
        properties: {
          originalRecord: {
            $ref: "#/components/schemas/Recordstringany",
          },
          errorMessage: {
            type: "string",
          },
          errorType: {
            type: "string",
          },
          failedAt: {
            type: "string",
            format: "date-time",
          },
          source: {
            oneOf: [
              {
                const: "api",
              },
              {
                const: "transform",
              },
              {
                const: "table",
              },
            ],
          },
        },
        required: [
          "originalRecord",
          "errorMessage",
          "errorType",
          "failedAt",
          "source",
        ],
      },
      Recordstringany: {
        type: "object",
        properties: {},
        required: [],
        description: "Construct a type with a set of properties K of type T",
        additionalProperties: {},
      },
    },
  },
  schemas: [
    {
      $ref: "#/components/schemas/DeadLetterModel",
    },
  ],
};

export const dlqColumns: Column[] = [
  {
    name: "originalRecord",
    data_type: "Json",
    primary_key: false,
    required: true,
    unique: false,
    default: null,
    annotations: [],
  },
  {
    name: "errorMessage",
    data_type: "String",
    primary_key: false,
    required: true,
    unique: false,
    default: null,
    annotations: [],
  },
  {
    name: "errorType",
    data_type: "String",
    primary_key: false,
    required: true,
    unique: false,
    default: null,
    annotations: [],
  },
  {
    name: "failedAt",
    data_type: "DateTime",
    primary_key: false,
    required: true,
    unique: false,
    default: null,
    annotations: [],
  },
  {
    name: "source",
    data_type: "String",
    primary_key: false,
    required: true,
    unique: false,
    default: null,
    annotations: [],
  },
];

export const getWorkflows = async () => {
  loadIndex();

  const registry = getMooseInternal();
  return registry.workflows;
};

function findTaskInTree(
  task: Task<any, any>,
  targetName: string,
): Task<any, any> | undefined {
  if (task.name === targetName) {
    return task;
  }

  if (task.config.onComplete?.length) {
    for (const childTask of task.config.onComplete) {
      const found = findTaskInTree(childTask, targetName);
      if (found) {
        return found;
      }
    }
  }

  return undefined;
}

export const getTaskForWorkflow = async (
  workflowName: string,
  taskName: string,
): Promise<Task<any, any>> => {
  const workflows = await getWorkflows();
  const workflow = workflows.get(workflowName);
  if (!workflow) {
    throw new Error(`Workflow ${workflowName} not found`);
  }

  const task = findTaskInTree(
    workflow.config.startingTask as Task<any, any>,
    taskName,
  );
  if (!task) {
    throw new Error(`Task ${taskName} not found in workflow ${workflowName}`);
  }

  return task;
};
