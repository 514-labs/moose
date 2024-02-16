import { CliData, DataModel, Route, Table } from "app/db";
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
  const routeTail = route.route_path.split("/").at(-1);
  return cliData.models.find((model) => model.name === routeTail);
}

export function tableIsIngestionTable(table: Table): boolean {
  return table.engine === "Kafka";
}

export function tableIsView(table: Table): boolean {
  return table.engine === "MaterializedView";
}

export function getQueueFromRoute(route: Route, cliData: CliData): string {
  const routeTail = route.route_path.split("/").at(-1);
  return cliData.queues.find((queue) => queue === routeTail);
}

export function getModelFromTable(table: Table, cliData: CliData): DataModel {
  if (table.engine === "MaterializedView") {
    const parsedViewName = table.name.split("_").at(0);
    return cliData.models.find((model) => model.name === parsedViewName);
  }

  return cliData.models.find((model) => model.name === table.name);
}

export function getRelatedInfra(
  model: DataModel,
  data: CliData,
  currectObject: any
): { tables: Table[]; ingestionPoints: Route[] } {
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
