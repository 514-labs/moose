import { column_type_mapper, is_enum } from "./utils";
import { CliData, DataModel, ModelMeta } from "app/types";

function createColumnStubs(model: ModelMeta) {
  return model.columns.map((field, index) => {
    if (is_enum(field.data_type)) {
      const value = JSON.stringify(field.data_type.Enum.values[0]);
      return `"${field.name}": ${value}`;
    }

    const data_type = column_type_mapper(field.data_type);
    switch (data_type) {
      case "number":
        return `"${field.name}": ${index}`;
      case "string":
        return `"${field.name}": "test-value${index}"`;
      case "boolean":
        return `"${field.name}": ${index % 2 === 0}`;
      case "Date":
        return `"${field.name}": "2022-01-01"`;
      case "DateTime":
        return `"${field.name}": "2024-02-20T23:14:57.788Z"`;
      case "array":
        return `"${field.name}": ["test-value${index}"]`;
      case "object":
        return `"${field.name}": { key: "test-value${index}" }`;
    }
  });
}
export const jsSnippet = (data: CliData, model: DataModel) => {
  const { ingestion_point } = model;
  const columns = createColumnStubs(model.model);

  if (!data.project || !ingestion_point) {
    return "no project found";
  }

  return `\
fetch('http://${data.project && data.project.http_server_config.host}:${data.project.http_server_config.port}/${ingestion_point.route_path}', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json'
    },
    body: JSON.stringify(
        {${columns.join(",")}}
    )
})
`;
};

export const pythonSnippet = (data: CliData, model: DataModel) => {
  const { ingestion_point } = model;
  const columns = createColumnStubs(model.model);

  if (!data.project || !ingestion_point) {
    return "no project found";
  }

  return `\
import requests

url = 'http://${data.project && data.project.http_server_config.host}:${data.project.http_server_config.port}/${ingestion_point.route_path}'
data = {
    ${columns.join(",")}
}
response = requests.post(url, json=data)
`;
};

export const clickhousePythonSnippet = (data: CliData, model: DataModel) => {
  /*
  const view = data.tables.find(
    (t) => t.name.includes(model.name) && t.engine === "MergeTree"
  );
  */
  const { table } = model;

  if (!table) {
    return "no view found";
  }

  return `\
import clickhouse_connect
import pandas

client = clickhouse_connect.get_client(
    host=${data.project && JSON.stringify(data.project.clickhouse_config.host)}
    port=${data.project && JSON.stringify(data.project.clickhouse_config.host_port)}
    user=${data.project && JSON.stringify(data.project.clickhouse_config.user)}
    password=${data.project && JSON.stringify(data.project.clickhouse_config.password)}
    database=${JSON.stringify(table.database)}
)

query_str = "SELECT * FROM ${table.name} LIMIT 10"

# query_df returns a dataframe
result = client.query_df(query_str)

print(result)
`;
};

export const clickhouseJSSnippet = (data: CliData, model: DataModel) => {
  const { table } = model;

  /*
  const view = data.tables.find(
    (t) => t.name.includes(model.name) && t.engine === "MergeTree"
  );
  */

  if (!table) {
    return "no view found";
  }

  return `import { createClient } from "@clickhouse/client-web"

const client = createClient({
    host: "http://${data.project && data.project.clickhouse_config.host}:${data.project && data.project.clickhouse_config.host_port}",
    username: ${data.project && JSON.stringify(data.project.clickhouse_config.user)},
    password: ${data.project && JSON.stringify(data.project.clickhouse_config.password)},
    database: ${JSON.stringify(table.database)},
});

const getResults = async () => {
    const resultSet = await client.query({
        query: "SELECT * FROM ${table.name} LIMIT 10",
        format: "JSONEachRow",
    });

    return resultSet.json();
};`;
};

export const curlSnippet = (data: CliData, model: DataModel) => {
  const { ingestion_point } = model;
  const columns = createColumnStubs(model.model);

  if (!data.project || !ingestion_point) {
    return "no project found";
  }

  return `\
curl -X POST -H "Content-Type: application/json" -d '{${columns.join()}}' \\
  http://${data.project && data.project.http_server_config.host}:${data.project.http_server_config.port}/${ingestion_point.route_path}
`;
};

export const bashSnippet = (data: CliData, model: DataModel) => {
  const curlCommand = curlSnippet(data, model);

  return `\
#!/bin/bash

for i in {1..10}; do
  ${curlCommand}
done
`;
};

export const rustSnippet = (data: CliData, model: DataModel) => {
  const { ingestion_point } = model;
  const columns = createColumnStubs(model.model);

  return `\
use reqwest::Client;
let client = Client::new();
let res = client.post("http://${data.project && data.project.http_server_config.host}:${data.project!.http_server_config.port}/${ingestion_point.route_path}")
    .json(&json!({${columns.join()}}))
    .send()
    .await?;
`;
};
