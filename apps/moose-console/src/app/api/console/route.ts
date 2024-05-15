import { putCliData } from "app/db";
import util from "util";

export async function POST(request: Request) {
  const json = await request.json();
  const log = util.inspect(json, {
    showHidden: false,
    depth: null,
    colors: true,
  });
  console.log("Received cli data:", log);
  await putCliData(json);
  return new Response("OK");
}
