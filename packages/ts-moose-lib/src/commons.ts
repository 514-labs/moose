import http from "http";
import { createClient } from "@clickhouse/client";

/**
 * Utility function for compiler-related logging that can be disabled via environment variable.
 * Set MOOSE_DISABLE_COMPILER_LOGS=true to suppress these logs (useful for testing environments).
 */
export const compilerLog = (message: string) => {
  if (process.env.MOOSE_DISABLE_COMPILER_LOGS !== "true") {
    console.log(message);
  }
};

export const antiCachePath = (path: string) =>
  `${path}?num=${Math.random().toString()}&time=${Date.now()}`;

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
    application: "moose",
    // Note: wait_end_of_query is configured per operation type, not globally
    // to preserve SELECT query performance while ensuring INSERT/DDL reliability
  });
};

export type CliLogData = {
  message_type?: "Info" | "Success" | "Error" | "Highlight";
  action: string;
  message: string;
};

export const cliLog: (log: CliLogData) => void = (log) => {
  const req = http.request({
    port: 5001,
    method: "POST",
    path: "/logs",
  });

  req.on("error", (err: Error) => {
    console.log(`Error ${err.name} sending CLI log.`, err.message);
  });

  req.write(JSON.stringify({ message_type: "Info", ...log }));
  req.end();
};

/**
 * Method to change .ts, .cts, and .mts to .js, .cjs, and .mjs
 * This is needed because 'import' does not support .ts, .cts, and .mts
 */
export function mapTstoJs(filePath: string): string {
  return filePath
    .replace(/\.ts$/, ".js")
    .replace(/\.cts$/, ".cjs")
    .replace(/\.mts$/, ".mjs");
}
