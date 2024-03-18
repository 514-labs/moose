import { createClient, ResultSet } from "npm:@clickhouse/client-web";

const CLICKHOUSE_HOST = Deno.env.get("CLICKHOUSE_HOST") || "clickhousedb";
const CLICKHOUSE_PORT = Deno.env.get("CLICKHOUSE_PORT") || "8123";
const CLICKHOUSE_USERNAME = Deno.env.get("CLICKHOUSE_USERNAME") || "panda";
const CLICKHOUSE_PASSWORD = Deno.env.get("CLICKHOUSE_PASSWORD") || "pandapass";
const CLICKHOUSE_DB = Deno.env.get("CLICKHOUSE_DB") || "local";

const clickhouse = createClient({
  host: `http://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
  username: CLICKHOUSE_USERNAME,
  password: CLICKHOUSE_PASSWORD,
  database: CLICKHOUSE_DB,
});

let i = 0;

const handler = async (request: Request): Response => {
  const path = new URL(request.url).pathname;

  // TODO: use static imports and have watcher recreate this file
  const userFuncModule = await import(
    // the path is different every time so it reloads
    `/scripts/${path}?dynamic_import_path_hack=${i++}`
  );

  const result = await userFuncModule.default(await request.json(), {
    clickhouse,
  });

  let body: string;
  if (result instanceof ResultSet) {
    body = JSON.stringify(await result.json());
  } else {
    body = JSON.stringify(result);
  }
  return new Response(body, { status: 200 });
};

Deno.serve({ port: 4001 }, handler);
