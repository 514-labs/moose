import process from "node:process";
import { ClickHouseClient } from "@clickhouse/client-web";
import fastq, { queueAsPromised } from "fastq";
import { cliLog, getClickhouseClient, walkDir } from "../commons";
import { Blocks } from "./helpers";


interface BlocksQueueTask {
  chClient: ClickHouseClient;
  blocks: Blocks;
  retries: number;
}

class DependencyError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "DependencyError";
  }
}

const [
  ,
  ,
  ,
  BLOCKS_DIR_PATH,
  CLICKHOUSE_DB,
  CLICKHOUSE_HOST,
  CLICKHOUSE_PORT,
  CLICKHOUSE_USERNAME,
  CLICKHOUSE_PASSWORD,
  CLICKHOUSE_USE_SSL,
] = process.argv;

const clickhouseConfig = {
  username: CLICKHOUSE_USERNAME,
  password: CLICKHOUSE_PASSWORD,
  database: CLICKHOUSE_DB,
  useSSL: CLICKHOUSE_USE_SSL,
  host: CLICKHOUSE_HOST,
  port: CLICKHOUSE_PORT,
};

const createBlocks = async (chClient: ClickHouseClient, blocks: Blocks) => {
  for (const query of blocks.setup) {
    try {
      console.log(`Creating block using query ${query}`);
      await chClient.command({ query });
    } catch (err) {
      cliLog({
        action: "Blocks",
        message: `Failed to create blocks: ${err}`,
        message_type: "Error",
      });
      if (err && JSON.stringify(err).includes(`UNKNOWN_TABLE`)) {
        throw new DependencyError(err.toString());
      }
    }
  }
};

const deleteBlocks = async (chClient: ClickHouseClient, blocks: Blocks) => {
  for (const query of blocks.teardown) {
    try {
      console.log(`Deleting block using query ${query}`);
      await chClient.command({ query });
    } catch (err) {
      cliLog({
        action: "Blocks",
        message: `Failed to delete blocks: ${err}`,
        message_type: "Error",
      });
    }
  }
};

const asyncWorker = async (task: BlocksQueueTask) => {
  await deleteBlocks(task.chClient, task.blocks);
  await createBlocks(task.chClient, task.blocks);
};

export const runBlocks = async () => {
  const chClient = getClickhouseClient(clickhouseConfig);
  console.log(`Connected`);

  const blocksFiles = walkDir(BLOCKS_DIR_PATH, ".ts", []);
  const numOfBlockFiles = blocksFiles.length;
  console.log(`Found ${numOfBlockFiles} blocks files`);

  const queue: queueAsPromised<BlocksQueueTask> = fastq.promise(asyncWorker, 1);

  queue.error((err: Error, task: BlocksQueueTask) => {
    if (err && task.retries > 0) {
      if (err instanceof DependencyError) {
        queue.push({ ...task, retries: task.retries - 1 });
      }
    }
  });

  for (const path of blocksFiles) {
    console.log(`Adding to queue: ${path}`);

    try {
      const blocks = require(path).default as Blocks;
      queue.push({
        chClient,
        blocks,
        retries: numOfBlockFiles,
      });
    } catch (err) {
      cliLog({
        action: "Blocks",
        message: `Failed to import blocks from ${path}: ${err}`,
        message_type: "Error",
      });
    }
  }

  while (!queue.idle()) {
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
};
