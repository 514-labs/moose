import { readProjectConfig } from "./configFile";

interface RuntimeClickHouseConfig {
  host: string;
  port: string;
  username: string;
  password: string;
  database: string;
  useSSL: boolean;
}

class ConfigurationRegistry {
  private static instance: ConfigurationRegistry;
  private clickhouseConfig?: RuntimeClickHouseConfig;

  static getInstance(): ConfigurationRegistry {
    if (!ConfigurationRegistry.instance) {
      ConfigurationRegistry.instance = new ConfigurationRegistry();
    }
    return ConfigurationRegistry.instance;
  }

  setClickHouseConfig(config: RuntimeClickHouseConfig): void {
    this.clickhouseConfig = config;
  }

  async getClickHouseConfig(): RuntimeClickHouseConfig {
    if (this.clickhouseConfig) {
      return this.clickhouseConfig;
    }

    // Fallback to reading from config file for backward compatibility
    const projectConfig = await readProjectConfig();
    return {
      host: projectConfig.clickhouse_config.host,
      port: projectConfig.clickhouse_config.host_port.toString(),
      username: projectConfig.clickhouse_config.user,
      password: projectConfig.clickhouse_config.password,
      database: projectConfig.clickhouse_config.db_name,
      useSSL: projectConfig.clickhouse_config.use_ssl || false,
    };
  }

  hasRuntimeConfig(): boolean {
    return !!this.clickhouseConfig;
  }
}

(globalThis as any)._mooseConfigRegistry = ConfigurationRegistry.getInstance();
export type { ConfigurationRegistry, RuntimeClickHouseConfig };
