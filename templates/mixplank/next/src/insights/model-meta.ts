import { getData } from "@/app/data";
import { EventTable } from "@/config";

// grabs column names and data types from all tables
// returning only the columns that are common to all tables

export function modelMeta(events: EventTable[]) {
  const tableNames = events.map((event) => `'${event.tableName}'`).join(", ");

  const query = `
  SELECT column_name, data_type
  FROM information_schema.columns 
  WHERE table_name IN (${tableNames})
  GROUP BY column_name, data_type
  HAVING COUNT(DISTINCT table_name) = ${events.length}
    `;
  return query;
}

export interface ModelMeta {
  column_name: string;
  data_type: string;
}

export function getModelMeta(events: EventTable[]) {
  if (events.length === 0) return Promise.resolve([]);
  return getData(modelMeta(events)) as Promise<ModelMeta[]>;
}
