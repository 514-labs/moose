import { ParsedLogs } from "../datamodels/logs";

export default function run(source: ParsedLogs): ParsedLogs {
  const centralTimeString = source.date;
  // Create a Date object from the central time string
  const centralDate = new Date(centralTimeString + " -0500"); // -0500 is the offset for Central Time Zone
  // Convert the Date object to UTC
  const utcDate = new Date(
    centralDate.getTime() + centralDate.getTimezoneOffset() * 60000,
  );

  return { ...source, date: utcDate };
}
