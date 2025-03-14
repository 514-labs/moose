import process from "process";
import { IngestApi, OlapTable, Stream } from "./index";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { Column } from "../dataModels/dataModelTypes";
import { IngestionFormat } from "../index";

const moose_internal = {
  tables: new Map<string, OlapTable<any>>(),
  streams: new Map<string, Stream<any>>(),
  apis: new Map<string, IngestApi<any>>(),
};
const defaultRetentionPeriod = 60 * 60 * 24 * 7;

interface TableJson {
  name: string;
  columns: Column[];
  order_by: string[];
  deduplicate: boolean;
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

  transformationTargets: Target[];
  hasMultiTransform: boolean;
}
interface IngestApiJson {
  name: string;
  columns: Column[];
  format: IngestionFormat;
  writeTo: Target;
}

const toInfraMap = (registry: typeof moose_internal) => {
  const tables: { [key: string]: TableJson } = {};
  const topics: { [key: string]: StreamJson } = {};
  const ingestApis: { [key: string]: IngestApiJson } = {};

  registry.tables.forEach((table) => {
    tables[table.name] = {
      name: table.name,
      columns: table.columnArray,
      order_by: table.config.order_by_fields ?? [],
      deduplicate: table.config.deduplicate ?? false,
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
    topics[stream.name] = {
      name: stream.name,
      columns: stream.columnArray,
      targetTable: stream.config.destination?.name,
      retentionPeriod: stream.config.retentionPeriod ?? defaultRetentionPeriod,
      partitionCount: stream.config.parallelism ?? 1,
      transformationTargets,
      hasMultiTransform: stream._multipleTransformations === undefined,
    };
  });

  registry.apis.forEach((api) => {
    ingestApis[api.name] = {
      name: api.name,
      columns: api.columnArray,
      format: api.config.format ?? IngestionFormat.JSON,
      writeTo: {
        kind: "stream",
        name: api.config.destination.name,
      },
    };
  });

  return {
    topics,
    tables,
    ingestApis,
  };
};

(globalThis as any).moose_internal = moose_internal;

export const getMooseInternal = (): typeof moose_internal =>
  (globalThis as any).moose_internal;

export const loadIndex = async () => {
  await require(`${process.cwd()}/app/index.ts`);

  console.log(
    "___MOOSE_STUFF___start",
    JSON.stringify(toInfraMap(getMooseInternal())),
    "end___MOOSE_STUFF___",
  );
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
