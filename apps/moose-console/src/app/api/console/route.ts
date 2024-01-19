import * as PouchDB from "pouchdb-node";
import { putCliData } from "../../db";

export async function POST(request: Request) {
  const json = await request.json();
  await putCliData(json);
  return new Response("OK");
}
