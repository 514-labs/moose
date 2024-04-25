"use server";
import { createClient } from "@clickhouse/client";

const clickhouseClient = createClient({
  host:
    process.env.DB_HOST ||
    "https://swtrnxdyro.us-central1.gcp.clickhouse.cloud:8443",
  username: process.env.DB_USER || "default",
  password: process.env.DB_PASS || "7On4.w79D84cn",
  database: process.env.DB || "default",
});

export const getData = async (query: string): Promise<object[]> => {
  if (!query) {
    return [];
  }
  const resultSet = await clickhouseClient.query({
    query,
    format: "JSONEachRow",
    clickhouse_settings: {
      output_format_json_quote_64bit_integers: 0,
    },
  });

  return await resultSet.json();
};

export const getMeta = async (query: string): Promise<object[]> => {
  if (!query) {
    return [];
  }
  const resultSet = await clickhouseClient.query({
    query,
    format: "JSON",
    clickhouse_settings: {
      output_format_json_quote_64bit_integers: 0,
    },
  });

  const { meta }: { meta: { name: string }[] } = await resultSet.json();
  return meta.map((m) => ({
    column_name: m.name,
  }));
};
