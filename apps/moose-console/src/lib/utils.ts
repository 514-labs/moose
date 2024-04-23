import {
  CliData,
  Table,
  CURRENT_VERSION,
  VersionKey,
  DataModel,
} from "app/types";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function column_type_mapper(source_type: string) {
  switch (source_type) {
    case "String":
      return "string";
    case "Float":
      return "float";
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

export function getModelsByVersion(
  cliData: CliData,
  version: VersionKey = CURRENT_VERSION,
) {
  if (version === CURRENT_VERSION) {
    return cliData.current.models;
  }
  console.log("CLIDATA", cliData);
  return cliData.past[version]!.models;
}

export function getModelByName(models: DataModel[], modelName: string) {
  return models.find((model) => model.model.name === modelName);
}
export function getModelByTableId(models: DataModel[], tableId: string) {
  const model = models.find((model) => model.table.uuid === tableId);
  if (model === undefined) throw new Error(`Model not found`);
  return model;
}

export function getModelByIngestionPointId(
  models: DataModel[],
  ingestionPointId: string,
) {
  const model = models.find(
    (model) =>
      model.ingestion_point.route_path.split("/").at(-2) === ingestionPointId,
  );
  if (model === undefined) throw new Error(`Model not found`);
  return model;
}

export function tableIsView(table: Table): boolean {
  return table.engine === "View";
}

export function tableIsIngestionTable(table: Table): boolean {
  return table.engine === "Kafka";
}

export function tableIsQueryable(table: Table): boolean {
  return table.engine === "MergeTree";
}

export function is_enum(type: any): type is { Enum: any } {
  return typeof type === "object" && type["Enum"] !== undefined;
}
