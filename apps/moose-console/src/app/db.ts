import { JSONFilePreset } from "lowdb/node";

// We added a DB here because we don't know how next will handle
// the request. We have found a hack where we could use modules to
// have a singleton and handle shared state but we are not guaranteed
// that it would work: https://stackoverflow.com/questions/13179109/singleton-pattern-in-nodejs-is-it-needed
// In terms of Local db implementation, we looked at: Level, PouchDB, sqlite3, lowdb

const DB_FILE = "./console.db";
const CLI_DATA_ID = "cliData";

const defaultData: {
  [CLI_DATA_ID]: CliData;
} = {
  [CLI_DATA_ID]: {
    current: {
      models: [],
    },
    past: {},
  },
};

export const CURRENT_VERSION = Symbol();
export type VersionKey = typeof CURRENT_VERSION | keyof VersionMap;

const dbPromise = JSONFilePreset(DB_FILE, defaultData);

export interface Route {
  file_path: string;
  route_path: string;
  table_name: string;
  view_name: string;
}

export interface Project {
  name: string;
  language: string;
  project_file_location: string;
  redpanda_config: RedpandaConfig;
  clickhouse_config: ClickhouseConfig;
  http_server_config: HTTPServerConfig;
  console_config: ConsoleConfig;
}

export interface RedpandaConfig {
  broker: string;
  message_timeout_ms: number;
}

export interface ClickhouseConfig {
  db_name: string;
  user: string;
  password: string;
  host: string;
  host_port: number;
  postgres_port: number;
  kafka_port: number;
}

export interface HTTPServerConfig {
  host: string;
  port: number;
}

export interface ConsoleConfig {
  host_port: number;
}

export interface ModelMeta {
  columns: Column[];
  name: string;
  db_name: string;
}

export interface DataModel {
  queue: string;
  table: Table;
  ingestion_point: Route;
  model: ModelMeta;
}

export interface MooseEnum {
  Enum: {
    name: string;
    values: string[];
  };
}

export interface Column {
  name: string;
  data_type: string | MooseEnum;
  arity: string;
  unique: boolean;
  primary_key: boolean;
  default: string;
}

export interface Table {
  database: string;
  dependencies_table: string[];
  engine: string;
  name: string;
  uuid: string;
}

export interface VersionMap {
  [version: string]: { models: DataModel[] };
}

export interface CliData {
  project?: Project;
  current: { models: DataModel[] };
  past: VersionMap;
}

export interface Infra {
  tables: Table[];
  ingestionPoints: Route[];
}

export function column_type_mapper(source_type: string) {
  switch (source_type) {
    case "String":
      return "string";
    case "Number":
      return "number";
    case "Boolean":
      return "boolean";
    case "Date":
      return "Date";
    case "DateTime":
      return "DateTime";
    case "Array":
      return "array";
    case "Object":
      return "object";
    default:
      return "unknown";
  }
}

export async function putCliData(data: CliData): Promise<void> {
  const db = await dbPromise;
  return await db.update((dbData) => {
    dbData[CLI_DATA_ID] = data;
  });
}

export async function getCliData(): Promise<CliData> {
  const db = await dbPromise;
  await db.read();
  console.log("db.data", db.data[CLI_DATA_ID]);
  return db.data[CLI_DATA_ID];
}
