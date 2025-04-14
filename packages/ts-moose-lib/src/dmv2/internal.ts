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

const moose_internal = {
  tables: new Map<string, OlapTable<any>>(),
  streams: new Map<string, Stream<any>>(),
  ingestApis: new Map<string, IngestApi<any>>(),
  egressApis: new Map<string, ConsumptionApi<any>>(),
  sqlResources: new Map<string, SqlResource>(),
};
const defaultRetentionPeriod = 60 * 60 * 24 * 7;

interface TableJson {
  name: string;
  columns: Column[];
  orderBy: string[];
  deduplicate: boolean;
  engine?: string;
  version?: string;
}
interface Target {
  name: string;
  kind: "stream"; // may add `| "table"` in the future
}
interface StreamJson {
  name: string;
  columns: Column[];
  retentionPeriod: number;
  partitionCount: number;
  targetTable?: string;
  targetTableVersion?: string;
  hasConsumers: boolean;
  version?: string;
  transformationTargets: Target[];
  hasMultiTransform: boolean;
}
interface IngestApiJson {
  name: string;
  columns: Column[];
  format: IngestionFormat;
  writeTo: Target;
  version?: string;
}

interface EgressApiJson {
  name: string;
  queryParams: Column[];
  responseSchema: IJsonSchemaCollection.IV3_1;
  version?: string;
}

interface SqlResourceJson {
  name: string;
  setup: readonly string[];
  teardown: readonly string[];
}

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

    stream._transformations.forEach(([destination, f]) => {
      transformationTargets.push({
        kind: "stream",
        name: destination.name,
      });
    });

    let hasConsumers = stream._consumers.length > 0;

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
      hasConsumers,
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

export const getMooseInternal = (): typeof moose_internal =>
  (globalThis as any).moose_internal;

// work around for variable visibility in compiler output
if (getMooseInternal() === undefined) {
  (globalThis as any).moose_internal = moose_internal;
}

export const loadIndex = async () => {
  await require(`${process.cwd()}/app/index.ts`);

  console.log(
    "___MOOSE_STUFF___start",
    JSON.stringify(toInfraMap(getMooseInternal())),
    "end___MOOSE_STUFF___",
  );
};

export const getStreamingFunctions = async () => {
  await require(`${process.cwd()}/app/index.ts`);

  const registry = getMooseInternal();
  const transformFunctions = new Map<string, (data: unknown) => unknown>();

  registry.streams.forEach((stream) => {
    stream._transformations.forEach(([destination, f]) => {
      // TODO: Add version to dmv2 apis
      const transformFunctionKey = `${stream.name}_${destination.name}`;
      transformFunctions.set(transformFunctionKey, f);
    });

    stream._consumers.forEach((consumer) => {
      // TODO: Add version to dmv2 apis
      const consumerFunctionKey = `${stream.name}_<no-target>`;
      transformFunctions.set(consumerFunctionKey, consumer);
    });
  });

  return transformFunctions;
};

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

export class TypedBase<T, C> {
  schema: IJsonSchemaCollection.IV3_1;
  name: string;

  columns: {
    [columnName in keyof T]: Column;
  };
  columnArray: Column[];

  config: C;

  /** @internal **/
  constructor(
    name: string,
    config: C,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    if (schema === undefined || columns === undefined) {
      throw new Error(
        "Supply the type param T so that the schema is inserted by the compiler plugin.",
      );
    }

    this.schema = schema;
    this.columnArray = columns;
    const columnsObj = {} as any;
    columns.forEach((column) => {
      columnsObj[column.name] = column;
    });
    this.columns = columnsObj;

    this.name = name;
    this.config = config;
  }
}
