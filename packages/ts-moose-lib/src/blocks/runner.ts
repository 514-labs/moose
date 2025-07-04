import { cliLog } from "../commons";
import fs from "node:fs";
import path from "node:path";

const walkDir = (dir: string, fileExtension: string, fileList: string[]) => {
  const files = fs.readdirSync(dir);

  files.forEach((file) => {
    if (fs.statSync(path.join(dir, file)).isDirectory()) {
      fileList = walkDir(path.join(dir, file), fileExtension, fileList);
    } else if (file.endsWith(fileExtension)) {
      fileList.push(path.join(dir, file));
    }
  });

  return fileList;
};

interface ClickhouseConfig {
  database: string;
  host: string;
  port: string;
  username: string;
  password: string;
  useSSL: boolean;
}

interface BlocksConfig {
  blocksDir: string;
  clickhouseConfig: ClickhouseConfig;
}

export const runBlocks = async (config: BlocksConfig) => {
  const blocksFiles = walkDir(config.blocksDir, ".ts", []);
  const numOfBlockFiles = blocksFiles.length;

  if (numOfBlockFiles > 0) {
    console.log(`⚠️  DEPRECATION WARNING: Blocks functionality has been deprecated and is no longer supported.`);
    console.log(`⚠️  Found ${numOfBlockFiles} blocks files in ${config.blocksDir}, but they will be ignored.`);
    console.log(`⚠️  Please migrate to the new data processing features. See documentation for alternatives.`);
    
    cliLog({
      action: "Blocks",
      message: `Blocks functionality is deprecated. Found ${numOfBlockFiles} blocks files but ignoring them.`,
      message_type: "Info",
    });
  } else {
    console.log(`No blocks files found in ${config.blocksDir}`);
  }

  // No-op: just return without doing anything
  return;
};
