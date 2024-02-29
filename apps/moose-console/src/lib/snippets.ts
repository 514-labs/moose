import { CliData, DataModel, column_type_mapper } from "app/db";
import { getIngestionPointFromModel, is_enum } from "./utils";

function createColumnStubs(model: DataModel) {
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
  const ingestionPoint = getIngestionPointFromModel(model, data);
  const columns = createColumnStubs(model);

  return `\
fetch('http://${data.project && data.project.http_server_config.host}:${data.project.http_server_config.port}/${ingestionPoint.route_path}', {
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
  return `\
import requests

url = 'http://${data.project && data.project.http_server_config.host}:${data.project.http_server_config.port}/${ingestionPoint.route_path}'
data = {
    ${columns.join(",")}
}
response = requests.post(url, json=data)
`;
};

export const clickhousePythonSnippet = (data: CliData, model: DataModel) => {
  const view = data.tables.find(
    (t) => t.name.includes(model.name) && t.engine === "MaterializedView",
  );

  return `\
import clickhouse_connect
import pandas

client = clickhouse_connect.get_client(
    host=${data.project && JSON.stringify(data.project.clickhouse_config.host)}
    port=${data.project && JSON.stringify(data.project.clickhouse_config.host_port)}
    user=${data.project && JSON.stringify(data.project.clickhouse_config.user)}
    password=${data.project && JSON.stringify(data.project.clickhouse_config.password)}
    database=${JSON.stringify(view.database)}
)

query_str = "SELECT * FROM ${view.name} LIMIT 10"

# query_df returns a dataframe
result = client.query_df(query_str)

print(result)
`;
};

export const clickhouseJSSnippet = (data: CliData, model: DataModel) => {
  const view = data.tables.find(
    (t) => t.name.includes(model.name) && t.engine === "MaterializedView",
  );

  return `import { createClient } from "@clickhouse/client-web"

const client = createClient({
    host: "http://${data.project && data.project.clickhouse_config.host}:${data.project && data.project.clickhouse_config.host_port}",
    username: ${data.project && JSON.stringify(data.project.clickhouse_config.user)},
    password: ${data.project && JSON.stringify(data.project.clickhouse_config.password)},
    database: ${JSON.stringify(view.database)},
});

const getResults = async () => {
    const resultSet = await client.query({
        query: "SELECT * FROM ${view.name} LIMIT 10",
        format: "JSONEachRow",
    });

    return resultSet.json();
};`;
};

export const curlSnippet = (data: CliData, model: DataModel) => {
  const ingestionPoint = getIngestionPointFromModel(model, data);
  const columns = createColumnStubs(model);

  return `\
curl -X POST -H "Content-Type: application/json" -d '{${columns.join()}}' \\
  http://${data.project && data.project.http_server_config.host}:${data.project.http_server_config.port}/${ingestionPoint.route_path}
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
