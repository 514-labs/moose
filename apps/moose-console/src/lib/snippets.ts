import { is_MooseInt, is_MooseString, is_enum } from "./utils";
import { CliData, DataModel, ModelMeta, MooseEnumMember } from "app/types";

function createColumnStubs(model: ModelMeta) {
  return model.columns.map((field, index) => {
    console.log(field);

    if (is_enum(field.data_type)) {
      const enumMember = field.data_type.values[0];

      if (enumMember) {
        if (!enumMember.value) {
          return `"${field.name}": 0`;
        } else if (is_MooseString(enumMember.value)) {
          return `"${field.name}": "${enumMember.value.String}"`;
        } else if (is_MooseInt(enumMember.value)) {
          return `"${field.name}": ${enumMember.value.Int}`;
        }
      }
    } else {
      const data_type = field.data_type.toLowerCase();
      if (data_type === "number") {
        return `"${field.name}": ${index}`;
      } else if (data_type === "float") {
        return `"${field.name}": ${index}.1`;
      } else if (data_type === "string") {
        return `"${field.name}": "test-value${index}"`;
      } else if (data_type === "boolean") {
        return `"${field.name}": ${index % 2 === 0}`;
      } else if (data_type === "date") {
        return `"${field.name}": "2022-01-01"`;
      } else if (data_type === "datetime") {
        return `"${field.name}": "2024-02-20T23:14:57.788Z"`;
      } else if (data_type.startsWith("array")) {
        return `"${field.name}": []`;
      }
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
