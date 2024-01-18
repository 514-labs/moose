/* eslint-disable turbo/no-undeclared-env-vars */

import { BaseResultSet, Row, createClient } from "@clickhouse/client-web";
import { unstable_noStore as noStore } from "next/cache";

async function getTable(tableName: string): Promise<any> {
  const CLICKHOUSE_HOST = process.env.CLICKHOUSE_HOST || "localhost";
  // Environment variables are always strings
  const CLICKHOUSE_PORT = process.env.CLICKHOUSE_PORT || "18123";
  const CLICKHOUSE_USERNAME = process.env.CLICKHOUSE_USERNAME || "panda";
  const CLICKHOUSE_PASSWORD = process.env.CLICKHOUSE_PASSWORD || "pandapass";
  const CLICKHOUSE_DB = process.env.CLICKHOUSE_DB || "local";

  const client = createClient({
    host: `http://${CLICKHOUSE_HOST}:${CLICKHOUSE_PORT}`,
    username: CLICKHOUSE_USERNAME,
    password: CLICKHOUSE_PASSWORD,
    database: CLICKHOUSE_DB,
  });

  const resultSet = await client.query({
    query: `SELECT * FROM ${tableName} LIMIT 10`,
    format: "JSONEachRow",
  });

  return resultSet.json();
}

interface TableProps {
  data: any[];
}

const Table = ({ data }: TableProps) => {
  // Get column headers (keys from the first object in the data array)
  const headers = data.length > 0 ? Object.keys(data[0]) : [];

  return (
    <table>
      <thead>
        <tr>
          {headers.map((header, index) => (
            <th key={index}>{header}</th>
          ))}
        </tr>
      </thead>
      <tbody>
        {data.map((row, rowIndex) => (
          <tr key={rowIndex}>
            {headers.map((header, cellIndex) => (
              <td key={cellIndex}>{row[header]}</td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
};

export default async function Page({
  params,
}: {
  params: { tableName: string };
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const tableData = await getTable(params.tableName);

  return (
    <div>
      <Table data={tableData} />
    </div>
  );
}
