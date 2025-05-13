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
  OlapTable,
  Stream,
  ConsumptionApi,
  SqlResource,
} from "./index";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { Column } from "../dataModels/dataModelTypes";
import { ConsumptionUtil, IngestionFormat } from "../index";

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
}
/**
 * JSON representation of an Ingest API configuration.
 */
interface IngestApiJson {
  /** The name of the Ingest API endpoint. */
  name: string;
  /** Array defining the expected input schema (columns/fields). */
  columns: Column[];
  /** The expected input data format (e.g., JSON). */
  format: IngestionFormat;
  /** The target stream where ingested data is written. */
  writeTo: Target;
  /** Optional version string for the API configuration. */
  version?: string;
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
const toInfraMap = (registry: typeof moose_internal) => {
  const tables: { [key: string]: TableJson } = {};
  const topics: { [key: string]: StreamJson } = {};
  const ingestApis: { [key: string]: IngestApiJson } = {};
  const egressApis: { [key: string]: EgressApiJson } = {};
  const sqlResources: { [key: string]: SqlResourceJson } = {};

  registry.tables.forEach((table) => {
    tables[table.name] = {
      name: table.name,
      columns: table.columnArray,
      orderBy: table.config.orderByFields ?? [],
      deduplicate: table.config.deduplicate ?? false,
      engine: table.config.engine,
      version: table.config.version,
    };
  });

  registry.streams.forEach((stream) => {
    const transformationTargets: Target[] = [];
    const consumers: Consumer[] = [];

    stream._transformations.forEach((transforms, destinationName) => {
      transforms.forEach(([destination, _, config]) => {
        transformationTargets.push({
          kind: "stream",
          name: destinationName,
          version: config.version,
        });
      });
    });

    stream._consumers.forEach((consumer) => {
      consumers.push({
        version: consumer.config.version,
      });
    });

    topics[stream.name] = {
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
    };
  });

  registry.ingestApis.forEach((api) => {
    ingestApis[api.name] = {
      name: api.name,
      columns: api.columnArray,
      format: api.config.format ?? IngestionFormat.JSON,
      version: api.config.version,
      writeTo: {
        kind: "stream",
        name: api.config.destination.name,
      },
    };
  });

  registry.egressApis.forEach((api) => {
    egressApis[api.name] = {
      name: api.name,
      queryParams: api.columnArray,
      responseSchema: api.responseSchema,
      version: api.config.version,
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

  return {
    topics,
    tables,
    ingestApis,
    egressApis,
    sqlResources,
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
export const loadIndex = async () => {
  await require(`${process.cwd()}/app/index.ts`);

  console.log(
    "___MOOSE_STUFF___start",
    JSON.stringify(toInfraMap(getMooseInternal())),
    "end___MOOSE_STUFF___",
  );
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
  await require(`${process.cwd()}/app/index.ts`);

  const registry = getMooseInternal();
  const transformFunctions = new Map<string, (data: unknown) => unknown>();

  registry.streams.forEach((stream) => {
    stream._transformations.forEach((transforms, destinationName) => {
      transforms.forEach(([_, transform, config]) => {
        const transformFunctionKey = `${stream.name}_${destinationName}${config.version ? `_${config.version}` : ""}`;
        console.log(`getStreamingFunctions: ${transformFunctionKey}`);
        transformFunctions.set(transformFunctionKey, transform);
      });
    });

    stream._consumers.forEach((consumer) => {
      const consumerFunctionKey = `${stream.name}_<no-target>${consumer.config.version ? `_${consumer.config.version}` : ""}`;
      transformFunctions.set(consumerFunctionKey, consumer.consumer);
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
  await require(`${process.cwd()}/app/index.ts`);
  const egressFunctions = new Map<
    string,
    (params: unknown, utils: ConsumptionUtil) => unknown
  >();

  const registry = getMooseInternal();
  registry.egressApis.forEach((api) => {
    egressFunctions.set(api.name, api.getHandler());
  });

  return egressFunctions;
};
