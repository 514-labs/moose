import { createClient } from "@clickhouse/client-web";
import { Project, getCliData } from "../app/db";
import { unstable_noStore as noStore } from "next/cache";

export function getClient(project?: Project) {
    // This is to make sure the environment variables are read at runtime
    // and not during build time
    noStore();

    console.log(project);
  
    const CLICKHOUSE_HOST = process.env.CLICKHOUSE_HOST || project?.clickhouse_config.host || "localhost";
    // Environment variables are always strings
    const CLICKHOUSE_PORT = process.env.CLICKHOUSE_PORT || project?.clickhouse_config.host_port || "18123";
    const CLICKHOUSE_USERNAME = process.env.CLICKHOUSE_USERNAME || project?.clickhouse_config.user ||"panda";
    const CLICKHOUSE_PASSWORD = process.env.CLICKHOUSE_PASSWORD || project?.clickhouse_config.password || "pandapass";
    const CLICKHOUSE_DB = "local";
  
    const client = createClient({
      host: `http://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
      username: CLICKHOUSE_USERNAME,
      password: CLICKHOUSE_PASSWORD,
      database: CLICKHOUSE_DB,
    });
  
    return client;
  }
  