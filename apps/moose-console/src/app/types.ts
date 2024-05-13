export const CURRENT_VERSION = "CURRENT_VERSION";
export type VersionKey = typeof CURRENT_VERSION | keyof VersionMap;

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
  flows: string[];
}

export type MooseColumnType = string | MooseEnum | MooseArrayType;

export interface MooseArrayType {
  elementType: MooseColumnType;
}

export interface MooseEnum {
  name: string;
  values: MooseEnumMember[];
}

export interface MooseEnumMember {
  name: string;
  value?: MooseInt | MooseString;
}

export type MooseInt = { Int: number };
export type MooseString = { String: string };

export interface Column {
  name: string;
  data_type: MooseColumnType;
  required: boolean;
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

type VersionMap = Record<string, { models: DataModel[] }>;

export interface CliData {
  project?: Project;
  current: { models: DataModel[] };
  past: VersionMap;
}

export interface Infra {
  tables: Table[];
  ingestionPoints: Route[];
}

export enum MooseObject {
  Model = "model",
  Table = "Table",
  IngestionPoint = "IngestionPoint",
  View = "View",
}

export const DB_FILE = "./console.db";
export const CLI_DATA_ID = "cliData";

export const defaultCliData: {
  [CLI_DATA_ID]: CliData;
} = {
  [CLI_DATA_ID]: {
    current: {
      models: [],
    },
    past: {},
  },
};
