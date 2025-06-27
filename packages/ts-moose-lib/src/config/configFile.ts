import path from "node:path";
import * as toml from "toml";

/**
 * ClickHouse configuration from moose.config.toml
 */
export interface ClickHouseConfig {
  host: string;
  host_port: number;
  user: string;
  password: string;
  db_name: string;
  use_ssl?: boolean;
  native_port?: number;
}

/**
 * Project configuration from moose.config.toml
 */
export interface ProjectConfig {
  language: string;
  clickhouse_config: ClickHouseConfig;
  // Add other config sections as needed
}

/**
 * Error thrown when configuration cannot be found or parsed
 */
export class ConfigError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ConfigError";
  }
}

/**
 * Walks up the directory tree to find moose.config.toml
 */
async function findConfigFile(
  startDir: string = process.cwd(),
): Promise<string | null> {
  const fs = await import("node:fs");

  let currentDir = path.resolve(startDir);

  while (true) {
    const configPath = path.join(currentDir, "moose.config.toml");
    if (fs.existsSync(configPath)) {
      return configPath;
    }

    const parentDir = path.dirname(currentDir);
    if (parentDir === currentDir) {
      // Reached root directory
      break;
    }
    currentDir = parentDir;
  }

  return null;
}

/**
 * Reads and parses the project configuration from moose.config.toml
 */
export async function readProjectConfig(): Promise<ProjectConfig> {
  const fs = await import("node:fs");
  const configPath = await findConfigFile();
  if (!configPath) {
    throw new ConfigError(
      "moose.config.toml not found in current directory or any parent directory",
    );
  }

  try {
    const configContent = fs.readFileSync(configPath, "utf-8");
    const config = toml.parse(configContent) as ProjectConfig;
    return config;
  } catch (error) {
    throw new ConfigError(`Failed to parse moose.config.toml: ${error}`);
  }
}
