import { CliData, DataModel, column_type_mapper } from "app/db";
import { getIngestionPointFromModel } from "./utils";

function createColumnStubs(model: DataModel) {
  return model.columns.map((field, index) => {
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
        return `"${field.name}": "2019-08-20 10:18:56"`;
      case "array":
        return `"${field.name}": ["test-value${index}"]`;
      case "object":
        return `"${field.name}": { key: "test-value${index}" }`;
    }
  });
}
export const jsSnippet = (data: CliData, model: DataModel) => {
  const ingestionPoint = getIngestionPointFromModel(model, data);

  const columns = createColumnStubs(model);

  return `
fetch('http://${data.project && data.project.local_webserver_config.host}:${data.project.local_webserver_config.port}/${ingestionPoint.route_path}', {
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
  const ingestionPoint = getIngestionPointFromModel(model, data);
  const columns = createColumnStubs(model);
  return `
import requests

url = 'http://${data.project && data.project.local_webserver_config.host}:${data.project.local_webserver_config.port}/${ingestionPoint.route_path}'
data = [
    ${columns.join(",")}
]
response = requests.post(url, json=data)
`;
};

export const clickhousePythonSnippet = (data: CliData, model: DataModel) => {
  const view = data.tables.find(
    (t) => t.name.includes(model.name) && t.engine === "MaterializedView",
  );

  return `
import clickhouse_connect

client = clickhouse_connect.get_client(
    host=${data.project && data.project.clickhouse_config.host}
    port=${data.project && data.project.clickhouse_config.host_port}
    user=${data.project && data.project.clickhouse_config.user}
    password=${data.project && data.project.clickhouse_config.password}
)

query_str = "SELECT * FROM ${view.name} LIMIT 10"
result = client.query(query_str)
print(result.result_rows)
`;
};

export const clickhouseJSSnippet = (data: CliData, model: DataModel) => {
  const view = data.tables.find(
    (t) => t.name.includes(model.name) && t.engine === "MaterializedView",
  );

  return `
import { createClient } from "@clickhouse/client-web"

const client = createClient({
    host: "http://${data.project && data.project.clickhouse_config.host}:${data.project && data.project.clickhouse_config.host_port}",
    username: ${data.project && data.project.clickhouse_config.user},
    password: ${data.project && data.project.clickhouse_config.password},
    database: ${view.database},
});

const resultSet = await client.query({
    query: "SELECT * FROM ${view.name} LIMIT 10",
    format: "JSONEachRow",
});

console.log(resultSet.json());
`;
};

export const curlSnippet = (data: CliData, model: DataModel) => {
  const ingestionPoint = getIngestionPointFromModel(model, data);
  const columns = createColumnStubs(model);

  return `
  curl -X POST -H "Content-Type: application/json" -d '{${columns.join()}}' \\
  http://${data.project && data.project.local_webserver_config.host}:${data.project.local_webserver_config.port}/${ingestionPoint.route_path}
`;
};

export const bashSnippet = (data: CliData, model: DataModel) => {
  const curlCommand = curlSnippet(data, model);

  return `
  #!/bin/bash

  for i in {1..10}; do
    ${curlCommand}
  done
  `;
};
