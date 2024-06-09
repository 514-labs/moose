import { getData } from "@/app/data";
import { EventTable } from "@/app/events";

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
  name: string;
  type: number;
}

export function getModelMeta() {
  const test = fetch("http://localhost:4000/consumption/event_attributes")
    .then((response) => response.json())
    .then((data: ModelMeta[]) => data)
    .catch((error) => console.error("Error:", error));

  return test;
}
