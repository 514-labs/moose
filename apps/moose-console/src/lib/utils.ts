import {
  CliData,
  Table,
  CURRENT_VERSION,
  VersionKey,
  DataModel,
  MooseEnum,
  MooseInt,
  MooseString,
  MooseColumnType,
  MooseArrayType,
} from "app/types";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
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
  const model = models.find((model) => model.table?.uuid === tableId);
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

export function isEnum(type: MooseColumnType): type is MooseEnum {
  return typeof type === "object" && Object.hasOwn(type, "name");
}
export function isArray(type: MooseColumnType): type is MooseArrayType {
  return typeof type === "object" && Object.hasOwn(type, "elementType");
}

export function isMooseInt(input: MooseInt | MooseString): input is MooseInt {
  return "Int" in input;
}

export function isMooseString(
  input: MooseInt | MooseString,
): input is MooseString {
  return "String" in input;
}
