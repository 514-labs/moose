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
    routes: [],
    tables: [],
    topics: [],
  },
};

const dbPromise = JSONFilePreset(DB_FILE, defaultData);

export interface Route {
  file_path: string;
  route_path: string;
  table_name: string;
  view_name: string;
}

export interface Table {
  database: string;
  dependencies_table: string[];
  engine: string;
  name: string;
  uuid: string;
}

export interface CliData {
  routes: Route[];
  tables: Table[];
  topics: string[];
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
