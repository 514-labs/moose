import fs from "node:fs";
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
function findConfigFile(startDir: string = process.cwd()): string | null {
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
export function readProjectConfig(): ProjectConfig {
  const configPath = findConfigFile();
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

/**
 * Reads the current project version from package.json
 */
export function getCurrentProjectVersion(): string {
  try {
    // Look for package.json in current directory first
    let packageJsonPath = path.join(process.cwd(), "package.json");

    if (!fs.existsSync(packageJsonPath)) {
      // Walk up to find package.json
      let currentDir = process.cwd();
      while (true) {
        packageJsonPath = path.join(currentDir, "package.json");
        if (fs.existsSync(packageJsonPath)) {
          break;
        }

        const parentDir = path.dirname(currentDir);
        if (parentDir === currentDir) {
          throw new ConfigError("package.json not found");
        }
        currentDir = parentDir;
      }
    }

    const packageContent = fs.readFileSync(packageJsonPath, "utf-8");
    const packageJson = JSON.parse(packageContent);

    if (!packageJson.version) {
      throw new ConfigError("No version field found in package.json");
    }

    return packageJson.version;
  } catch (error) {
    throw new ConfigError(`Failed to read project version: ${error}`);
  }
}

/**
 * Generates the versioned table name following Moose's naming convention
 * Format: {tableName}_{version_with_dots_replaced_by_underscores}
 */
export function generateTableName(baseName: string, version?: string): string {
  const projectVersion = version || getCurrentProjectVersion();
  const versionSuffix = projectVersion.replace(/\./g, "_");
  return `${baseName}_${versionSuffix}`;
}
