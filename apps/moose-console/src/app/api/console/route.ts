import { putCliData } from "../../db";

export async function POST(request: Request) {
  const json = await request.json();
  console.log("Received cli data", json);
  await putCliData(json);
  return new Response("OK");
}
