import { parse } from "csv-parse";
import { queue } from "async";
import { ParsedLogs } from "./app/datamodels/logs";
import * as fs from "fs";

const csvParser = parse({
  delimiter: ",",
  columns: true,
  comment: "#",
});

async function sendParsedLogs(parsedLogs: ParsedLogs) {
  parsedLogs.date = new Date();

  return fetch(`http://localhost:4000/ingest/ParsedLogs/0.5`, {
    method: "POST",
    mode: "no-cors",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(parsedLogs),
  });
}

const uploadQueue = queue(async (record: ParsedLogs) => {
  let res = await sendParsedLogs(record);

  if (!res.ok) {
    console.error(`Response Status: ${res.status}`);
  }
}, 1000);

const reader = fs.createReadStream(
  "/Users/georgeanderson/Downloads/parsed_logs.csv",
);
reader.pipe(csvParser);

csvParser.on("readable", async function () {
  let record;
  while ((record = csvParser.read()) !== null) {
    uploadQueue.push(record);
  }

  uploadQueue.drain().then(() => {
    console.log("Records uploaded");
  });
});
