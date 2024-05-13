import {
  createClient,
  ClickHouseClient,
} from "npm:@clickhouse/client-web@1.0.1";
import { watch } from "npm:chokidar@3.6.0";
import * as fastq from "npm:fastq@1.17.1";
import type { queueAsPromised } from "npm:fastq@1.17.1";
import { existsSync, walkSync } from "https://deno.land/std@0.224.0/fs/mod.ts";

interface ShowTablesResponse {
  meta: { name: string; type: string }[];
  data: { name: string }[];
  rows: number;
}

interface MvQuery {
  select: string;
  orderBy: string;
}

interface MvQueueTask {
  event: "add" | "unlink" | "change";
  chClient: ClickHouseClient;
  path: string;
  retries: number;
}

class DependencyError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "DependencyError";
  }
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
    CLICKHOUSE_USE_SSL === "1" || CLICKHOUSE_USE_SSL.toLowerCase() === "true"
      ? "https"
      : "http";
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

const countAggregations = () => {
  let count = 0;

  for (const _ of walkSync(AGGREGATIONS_DIR_PATH, {
    includeDirs: false,
    exts: ["ts"],
  })) {
    count++;
  }

  return count;
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

    if (
      err &&
      err.toString().includes(`${AGGREGATIONS_MV_SUFFIX} does not exist`)
    ) {
      throw new DependencyError(err);
    }
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

async function asyncWorker(task: MvQueueTask): Promise<void> {
  if (task.event === "add") {
    await createAggregation(task.chClient, task.path);
  } else if (task.event === "unlink") {
    await deleteAggregation(task.chClient, task.path);
  } else if (task.event === "change") {
    await deleteAggregation(task.chClient, task.path);
    await createAggregation(task.chClient, task.path);
  }
}

const startFileWatcher = async (chClient: ClickHouseClient) => {
  const pathToWatch = `${AGGREGATIONS_DIR_PATH}/**/${AGGREGATIONS_FILE}`;
  const queue: queueAsPromised<MvQueueTask> = fastq.promise(asyncWorker, 1);
  const numOfAggregations = countAggregations();
  console.log(`Found ${numOfAggregations} aggregations`);

  queue.error((err: Error, task: MvQueueTask) => {
    if (err && task.retries > 0) {
      if (err instanceof DependencyError) {
        queue.push({ ...task, retries: task.retries - 1 });
      }
    }
  });

  watch(pathToWatch, { usePolling: true }).on(
    "all",
    (event: string, path: string) => {
      console.log(`Adding to queue: ${event} ${path}`);
      const antiCachePath = `${path}?num=${Math.random().toString()}&time=${Date.now()}`;
      queue.push({
        event: event as MvQueueTask["event"],
        chClient,
        path: antiCachePath,
        retries: numOfAggregations,
      });
    },
  );

  console.log(`Watching for changes to ${pathToWatch}...`);
};

const main = async () => {
  if (!existsSync(AGGREGATIONS_DIR_PATH)) {
    console.log(`${AGGREGATIONS_DIR_PATH} not found. Exiting...`);
    Deno.exit(1);
  }

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
