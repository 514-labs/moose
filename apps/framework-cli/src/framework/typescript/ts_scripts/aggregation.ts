import process from "node:process";
import { ClickHouseClient } from "@clickhouse/client-web";
import fastq, { queueAsPromised } from "fastq";
import {
  getFileName,
  walkDir,
  getClickhouseClient,
  cliLog,
} from "@514labs/moose-lib";

interface MvQuery {
  select: string;
  orderBy: string;
}

interface MvQueueTask {
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

const AGGREGATIONS_DIR_PATH = process.argv[1];

const [
  ,
  ,
  CLICKHOUSE_DB,
  CLICKHOUSE_HOST,
  CLICKHOUSE_PORT,
  CLICKHOUSE_USERNAME,
  CLICKHOUSE_PASSWORD,
  CLICKHOUSE_USE_SSL,
] = process.argv;

export const clickhouseConfig = {
  username: CLICKHOUSE_USERNAME,
  password: CLICKHOUSE_PASSWORD,
  database: CLICKHOUSE_DB,
  useSSL: CLICKHOUSE_USE_SSL,
  host: CLICKHOUSE_HOST,
  port: CLICKHOUSE_PORT,
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
            CREATE MATERIALIZED VIEW IF NOT EXISTS ${fileName}
            ENGINE = AggregatingMergeTree() ORDER BY ${mvObj.orderBy}
            POPULATE
            AS ${mvObj.select}
        `;
    await chClient.command({ query: mvQuery });
    console.log(`Created aggregation ${fileName}. Query: ${mvQuery}`);
    cliLog({ action: "Created", message: `aggregation ${fileName}` });
  } catch (err) {
    console.error(`Failed to create aggregation ${fileName}: ${err}`);
    cliLog({ action: "Failed", message: `to create aggregation ${fileName}` });

    if (err && JSON.stringify(err).includes(`UNKNOWN_TABLE`)) {
      throw new DependencyError(err.toString());
    }
  }
};

const deleteAggregation = async (chClient: ClickHouseClient, path: string) => {
  const fileName = getFileName(path);

  try {
    await chClient.command({
      query: `DROP VIEW IF EXISTS ${fileName}`,
    });
    console.log(`Deleted aggregation ${fileName}`);
    cliLog({ action: "Deleted", message: `aggregation ${fileName}` });
  } catch (err) {
    console.error(`Failed to delete aggregation ${fileName}: ${err}`);
  }
};

const asyncWorker = async (task: MvQueueTask) => {
  await deleteAggregation(task.chClient, task.path);
  await createAggregation(task.chClient, task.path);
};

const main = async () => {
  const chClient = getClickhouseClient(clickhouseConfig);
  console.log(`Connected`);

  const aggregationFiles = walkDir(AGGREGATIONS_DIR_PATH, ".ts", []);
  const numOfAggregations = aggregationFiles.length;
  console.log(`Found ${numOfAggregations} aggregations`);

  const queue: queueAsPromised<MvQueueTask> = fastq.promise(asyncWorker, 1);

  queue.error((err: Error, task: MvQueueTask) => {
    if (err && task.retries > 0) {
      if (err instanceof DependencyError) {
        queue.push({ ...task, retries: task.retries - 1 });
      }
    }
  });

  for (const path of aggregationFiles) {
    console.log(`Adding to queue: ${path}`);
    queue.push({
      chClient,
      path,
      retries: numOfAggregations,
    });
  }

  while (!queue.idle()) {
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
};

main();
