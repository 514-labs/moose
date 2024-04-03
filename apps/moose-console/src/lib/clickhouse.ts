"use server";

import { createClient } from "@clickhouse/client-web";
import { Project } from "app/types";
import { sendServerEvent } from "event-capture/server-event";

function getClient(project?: Project) {
  const CLICKHOUSE_HOST =
    process.env.CLICKHOUSE_HOST ||
    project?.clickhouse_config.host ||
    "localhost";
  // Environment variables are always strings
  const CLICKHOUSE_PORT =
    process.env.CLICKHOUSE_PORT ||
    project?.clickhouse_config.host_port ||
    "18123";
  const CLICKHOUSE_USERNAME =
    process.env.CLICKHOUSE_USERNAME ||
    project?.clickhouse_config.user ||
    "panda";
  const CLICKHOUSE_PASSWORD =
    process.env.CLICKHOUSE_PASSWORD ||
    project?.clickhouse_config.password ||
    "pandapass";
  const CLICKHOUSE_DB = "local";

  const client = createClient({
    host: `http://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
    username: CLICKHOUSE_USERNAME,
    password: CLICKHOUSE_PASSWORD,
    database: CLICKHOUSE_DB,
  });

  return client;
}

export async function runQuery(
  project: Project,
  queryString: string,
): Promise<any> {
  const client = getClient(project);
  const resultSet = await client
    .query({
      query: queryString,
      format: "JSONEachRow",
    })
    .then(
      (res) => {
        sendServerEvent("Query Success", { query: queryString });
        return res.json();
      },
      (_e) => {
        sendServerEvent("Query Error", { query: queryString });
        return [];
      },
    );

  return resultSet;
}
