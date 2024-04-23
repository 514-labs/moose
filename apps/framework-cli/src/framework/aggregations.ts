import {
  createClient,
  ClickHouseClient,
} from "npm:@clickhouse/client-web@1.0.1";
import { watch } from "npm:chokidar@3.6.0";

interface ShowTablesResponse {
  meta: { name: string; type: string }[];
  data: { name: string }[];
  rows: number;
}

const cwd = Deno.args[0] || Deno.cwd();
const AGGREGATIONS_DIR_PATH = `${cwd}/app/aggregations`;
const AGGREGATIONS_FILE = "*.ts";

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
    url: `${protocol}://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
    username: CLICKHOUSE_USERNAME,
    password: CLICKHOUSE_PASSWORD,
    database: CLICKHOUSE_DB,
  });
};

const waitForClickhouse = (chClient: ClickHouseClient) => {
  const pingClickhouse = async () => {
    try {
      const queryRespose = await chClient.query({ query: "SHOW TABLES" });
      const showTablesResponse =
        (await queryRespose.json()) as ShowTablesResponse;
      return showTablesResponse.rows > 0;
    } catch (_) {
      return false;
    }
  };

  const poll = (
    resolve: (value?: unknown) => void,
    reject: (reason?: any) => void,
  ) => {
    pingClickhouse().then((condition) => {
      if (condition) resolve();
      else setTimeout(() => poll(resolve, reject), 1000);
    });
  };

  return new Promise(poll);
};

const getVersion = () => {
  const version = JSON.parse(Deno.readTextFileSync(`${cwd}/package.json`))
    .version as string;
  return version.replace(/\./g, "_");
};

const getFileName = (filePath: string) => {
  const regex = /\/([^\/]+)\.ts/;
  const matches = filePath.match(regex);
  if (matches && matches.length > 1) {
    return matches[1];
  }
  return "";
};

const createAggregation = async (
  chClient: ClickHouseClient,
  path: string,
  version: string,
) => {
  const fileName = getFileName(path);

  try {
    const sqlString = (await import(path)).default;
    if (typeof sqlString !== "string") {
      console.error(
        `Not creating aggregation. Expected an export default SQL string from ${fileName}`,
      );
      return;
    }

    const mvQuery = `
          CREATE MATERIALIZED VIEW IF NOT EXISTS ${fileName}Mv
          TO ${fileName}_${version}
          AS ${sqlString}
      `;
    await chClient.command({ query: mvQuery });
    console.log(`Created aggregation ${fileName}`);
  } catch (err) {
    console.error(`Failed to create aggregation ${fileName}: ${err}`);
  }
};

const deleteAggregation = async (
  chClient: ClickHouseClient,
  path: string,
  version: string,
) => {
  const fileName = getFileName(path);

  try {
    await chClient.command({ query: `DROP VIEW IF EXISTS ${fileName}Mv` });
    await chClient.command({
      query: `DROP TABLE IF EXISTS ${fileName}_${version}`,
    });
    console.log(`Deleted aggregation ${fileName}`);
  } catch (err) {
    console.error(`Failed to delete aggregation ${fileName}: ${err}`);
  }
};

const startFileWatcher = (chClient: ClickHouseClient) => {
  const version = getVersion();
  const pathToWatch = `${AGGREGATIONS_DIR_PATH}/**/${AGGREGATIONS_FILE}`;

  watch(pathToWatch, { usePolling: true }).on(
    "all",
    async (event: string, path: string) => {
      const antiCachePath = `${path}?num=${Math.random().toString()}&time=${Date.now()}`;

      if (event === "add") {
        await createAggregation(chClient, antiCachePath, version);
      } else if (event === "unlink") {
        await deleteAggregation(chClient, antiCachePath, version);
      } else if (event === "change") {
        await deleteAggregation(chClient, antiCachePath, version);
        await createAggregation(chClient, antiCachePath, version);
      }
    },
  );

  console.log(`Watching for changes to ${pathToWatch}...`);
};

const main = async () => {
  const chClient = getClickhouseClient();
  await waitForClickhouse(chClient);

  console.log(`Connected`);
  startFileWatcher(chClient);
};

main().catch((err) => {
  console.error(err);
  Deno.exit(1);
});
