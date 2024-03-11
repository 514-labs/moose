import { CliData, DataModel, Infra, Route, Table } from "app/db";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function getIngestionPointFromModel(
  model: DataModel,
  cliData: CliData
): Route {
  return cliData.ingestionPoints.find((ingestionPoint) =>
    ingestionPoint.route_path.includes(model.name)
  );
}

export function getModelFromRoute(route: Route, cliData: CliData): DataModel {
  const routeTail = route.route_path.split("/").at(-2); // -1 is now version number
  const found = cliData.models.find((model) => model.name === routeTail);
  if (found === undefined) throw new Error(`Model ${routeTail} not found`);
  return found;
}

export function tableIsIngestionTable(table: Table): boolean {
  return table.engine === "Kafka";
}

export function tableIsQueryable(table: Table): boolean {
  return table.engine === "MergeTree";
}

export function getQueueFromRoute(route: Route, cliData: CliData): string {
  const routeTail = route.route_path.split("/").at(-1);
  return cliData.queues.find((queue) => queue === routeTail);
}

export function getModelFromTable(table: Table, cliData: CliData): DataModel {
  // TODO: this breaks if the model name includes underscore(s)
  // maybe include more information in `CliData`, so we don't have to lookup by name
  const table_name = table.name.split("_").at(0);

  const result = cliData.models.find((model) => model.name === table_name);
  if (result === undefined) throw new Error(`Model ${table_name} not found`);
  return result;
}

export function getRelatedInfra(
  model: DataModel,
  data: CliData,
  currectObject: any
): Infra {
  const tables = data.tables.filter(
    (t) => t.name.includes(model.name) && t.uuid !== currectObject.uuid
  );
  const ingestionPoints = data.ingestionPoints.filter(
    (ip) =>
      ip.route_path.includes(model.name) &&
      ip.route_path !== currectObject.route_path
  );

  return { tables, ingestionPoints };
}

export function is_enum(type: any): type is { Enum: any } {
  return typeof type === "object" && type["Enum"] !== undefined;
}
