import { createClient } from "@clickhouse/client-web";
import fs from "node:fs";
import path from "node:path";
import http from "http";

export const antiCachePath = (path: string) =>
  `${path}?num=${Math.random().toString()}&time=${Date.now()}`;

export const walkDir = (
  dir: string,
  fileExtension: string,
  fileList: string[],
) => {
  const files = fs.readdirSync(dir);

  files.forEach((file) => {
    if (fs.statSync(path.join(dir, file)).isDirectory()) {
      fileList = walkDir(path.join(dir, file), fileExtension, fileList);
    } else if (file.endsWith(fileExtension)) {
      fileList.push(path.join(dir, file));
    }
  });

  return fileList;
};

export const getFileName = (filePath: string) => {
  const regex = /\/([^\/]+)\.ts/;
  const matches = filePath.match(regex);
  if (matches && matches.length > 1) {
    return matches[1];
  }
  return "";
};

interface ClientConfig {
  username: string;
  password: string;
  database: string;
  useSSL: string;
  host: string;
  port: string;
}

export const getClickhouseClient = ({
  username,
  password,
  database,
  useSSL,
  host,
  port,
}: ClientConfig) => {
  const protocol =
    useSSL === "1" || useSSL.toLowerCase() === "true" ? "https" : "http";
  console.log(`Connecting to Clickhouse at ${protocol}://${host}:${port}`);
  return createClient({
    url: `${protocol}://${host}:${port}`,
    username: username,
    password: password,
    database: database,
  });
};

type CliLogData = {
  message_type?: "Info" | "Success" | "Error" | "Highlight";
  action: string;
  message: string;
};
export const cliLog: (log: CliLogData) => void = (log) => {
  const req = http.request({
    port: 5000,
    method: "POST",
    path: "/logs",
  }); // no callback, fire and forget

  req.write(JSON.stringify({ message_type: "Info", ...log }));
  req.end();
};
