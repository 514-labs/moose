import {
  createClient,
  ClickHouseClient,
  ResultSet,
} from "npm:@clickhouse/client-web@1.0.1";

const CLICKHOUSE_DB =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__DB_NAME") || "local";
const CLICKHOUSE_HOST =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__HOST") || "localhost";
const CLICKHOUSE_PORT =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__HOST_PORT") || "18123";
const CLICKHOUSE_USERNAME =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__USER") || "panda";
const CLICKHOUSE_PASSWORD =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__PASSWORD") || "pandapass";
const CLICKHOUSE_USE_SSL =
  Deno.env.get("MOOSE_CLICKHOUSE_CONFIG__USE_SSL") || "false";

const getClickhouseClient = () => {
  const protocol =
    CLICKHOUSE_USE_SSL.toLowerCase() === "true" ? "https" : "http";
  console.log(
    `Connecting to Clickhouse at ${protocol}://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
  );
  return createClient({
    host: `${protocol}://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
    username: CLICKHOUSE_USERNAME,
    password: CLICKHOUSE_PASSWORD,
    database: CLICKHOUSE_DB,
  });
};

let i = 0;
const cwd = Deno.args[0] || Deno.cwd();
const FLOWS_DIR_PATH = `${cwd}/app/apis`;

const getFlows = (): Map<string, Map<string, string>> => {
  const flowsDir = Deno.readDirSync(FLOWS_DIR_PATH);
  const output = new Map<string, Map<string, string>>();

  for (const source of flowsDir) {
    if (!source.isDirectory) continue;

    const flows = new Map<string, string>();
    const destinations = Deno.readDirSync(`${FLOWS_DIR_PATH}/${source.name}`);
    for (const destination of destinations) {
      if (!destination.isDirectory) continue;

      const destinationFiles = Deno.readDirSync(
        `${FLOWS_DIR_PATH}/${source.name}/${destination.name}`,
      );
      for (const destinationFile of destinationFiles) {
        if (destinationFile.isFile) {
          flows.set(
            destination.name,
            `${FLOWS_DIR_PATH}/${source.name}/${destination.name}/${destinationFile.name}`,
          );
        }
      }
    }

    if (flows.size > 0) {
      output.set(source.name, flows);
    }
  }

  return output;
};

const apiHandler = async (request: Request): Promise<Response> => {
  console.log("Request received", request);
  const path = request.url;

  // TODO: use static imports and have watcher recreate this file
  const userFuncModule = await import(
    // the path is different every time so it reloads
    `/apis/${path}?dynamic_import_path_hack=${i++}`
  );

  const result = await userFuncModule.default(
    await request,
    getClickhouseClient(),
  );

  let body: string;
  if (result instanceof ResultSet) {
    body = JSON.stringify(await result.json());
  } else {
    body = JSON.stringify(result);
  }
  return new Response(body, { status: 200 });
};

export const startApiService = async () =>
  Deno.serve({ port: 4001 }, apiHandler);

startApiService();
