"use server";
import { createClient } from "@clickhouse/client";
import { column } from "@observablehq/plot";

const clickhouseClient = createClient({
  host: "http://localhost:18123",
  username: "panda",
  password: "pandapass",
  database: "local",
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
  console.log(meta, "meta");
  return meta.map((m) => ({
    column_name: m.name,
  }));
};
