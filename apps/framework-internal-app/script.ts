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
  const centralTimeString = parsedLogs.date;
  // Create a Date object from the central time string
  const centralDate = new Date(centralTimeString + " -0500"); // -0500 is the offset for Central Time Zone
  // Convert the Date object to UTC
  const utcDate = new Date(
    centralDate.getTime() + centralDate.getTimezoneOffset() * 60000,
  );

  return fetch(`http://localhost:4000/ingest/ParsedLogs/0.6`, {
    method: "POST",
    mode: "no-cors",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ ...parsedLogs, date: utcDate }),
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
