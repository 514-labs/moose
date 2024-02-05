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
    models: [],
    ingestionPoints: [],
    tables: [],
    queues: [],
  },
};

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
  local_webserver_config: LocalWebserverConfig;
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
  cluster_network: string;
}

export interface LocalWebserverConfig {
  host: string;
  port: number;
}

export interface ConsoleConfig {
  host_port: number;
}


export interface DataModel {
  db_name: string;
  columns: Column[];
  name: string;
  version: number;
}

export interface Column {
  name: string;
  data_type: string;
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

export interface CliData {
  project?: Project;
  models: DataModel[];
  ingestionPoints: Route[];
  tables: Table[];
  queues: string[];
}

export function column_type_mapper(source_type: string): string {
  if (source_type === "String") {
    return "string";
  } else if (source_type === "Number") {
    return "number";
  } else if (source_type === "Boolean") {
    return "boolean";
  } else if (source_type === "Date") {
    return "Date";
  } else if (source_type === "Array") {
    return "array";
  } else if (source_type === "Object") {
    return "object";
  } else {
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
  return db.data[CLI_DATA_ID];
}
