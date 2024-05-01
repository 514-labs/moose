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

interface MvQuery {
  select: string;
  orderBy: string;
}

const cwd = Deno.args[0] || Deno.cwd();
const AGGREGATIONS_DIR_PATH = `${cwd}/app/aggregations`;
const AGGREGATIONS_FILE = "*.ts";
const AGGREGATIONS_MV_SUFFIX = "aggregations_mv";

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

const getFileName = (filePath: string) => {
  const regex = /\/([^\/]+)\.ts/;
  const matches = filePath.match(regex);
  if (matches && matches.length > 1) {
    return matches[1];
  }
  return "";
};

const cleanUpAggregations = async (chClient: ClickHouseClient) => {
  try {
    console.log("Preparing aggregations");
    const query = `SHOW TABLES LIKE '%${AGGREGATIONS_MV_SUFFIX}'`;
    const queryResponse = await chClient.query({ query });
    const showTablesResponse =
      (await queryResponse.json()) as ShowTablesResponse;

    showTablesResponse.data.forEach(async (table) => {
      await chClient.command({
        query: `DROP VIEW IF EXISTS ${table.name}`,
      });
    });
  } catch (err) {
    console.error(`Failed to clean up aggregations: ${err}`);
  }
};

const createAggregation = async (chClient: ClickHouseClient, path: string) => {
  const fileName = getFileName(path);

  try {
    const mvObj = (await import(path)).default as MvQuery;

    if (!mvObj.select || typeof mvObj.select !== "string") {
      throw new Error("Aggregation select query needs to be a string");
    }
    if (!mvObj.orderBy || typeof mvObj.orderBy !== "string") {
      throw new Error("Aggregation orderBy field needs to be a string");
    }

    const mvQuery = `
          CREATE MATERIALIZED VIEW IF NOT EXISTS ${fileName}_${AGGREGATIONS_MV_SUFFIX}
          ENGINE = AggregatingMergeTree() ORDER BY ${mvObj.orderBy}
          POPULATE
          AS ${mvObj.select}
      `;
    await chClient.command({ query: mvQuery });
    console.log(`Created aggregation ${fileName}`);
  } catch (err) {
    console.error(`Failed to create aggregation ${fileName}: ${err}`);
  }
};

const deleteAggregation = async (chClient: ClickHouseClient, path: string) => {
  const fileName = getFileName(path);

  try {
    await chClient.command({
      query: `DROP VIEW IF EXISTS ${fileName}_${AGGREGATIONS_MV_SUFFIX}`,
    });
    console.log(`Deleted aggregation ${fileName}`);
  } catch (err) {
    console.error(`Failed to delete aggregation ${fileName}: ${err}`);
  }
};

const startFileWatcher = (chClient: ClickHouseClient) => {
  const pathToWatch = `${AGGREGATIONS_DIR_PATH}/**/${AGGREGATIONS_FILE}`;

  watch(pathToWatch, { usePolling: true }).on(
    "all",
    async (event: string, path: string) => {
      const antiCachePath = `${path}?num=${Math.random().toString()}&time=${Date.now()}`;

      if (event === "add") {
        await createAggregation(chClient, antiCachePath);
      } else if (event === "unlink") {
        await deleteAggregation(chClient, antiCachePath);
      } else if (event === "change") {
        await deleteAggregation(chClient, antiCachePath);
        await createAggregation(chClient, antiCachePath);
      }
    },
  );

  console.log(`Watching for changes to ${pathToWatch}...`);
};

const main = async () => {
  const chClient = getClickhouseClient();
  await waitForClickhouse(chClient);

  console.log(`Connected`);
  await cleanUpAggregations(chClient);
  startFileWatcher(chClient);
};

main().catch((err) => {
  console.error(err);
  Deno.exit(1);
});
