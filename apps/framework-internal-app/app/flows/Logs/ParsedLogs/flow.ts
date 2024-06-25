// Example flow function: Converts local timestamps in UserActivity data to UTC.

// Imports: Source (UserActivity) and Destination (ParsedActivity) data models.
import { ParsedLogs, Logs } from "../../../datamodels/logs";

// The 'run' function transforms UserActivity data to ParsedActivity format.
// For more details on how Moose flows work, see: https://docs.moosejs.com
export default function run(source: Logs): ParsedLogs {
  // Convert local timestamp to UTC and return new ParsedActivity object.
  return {
    date: new Date(
      source.resourceLogs[0].scopeLogs[0].logRecords[0].observedTimeUnixNano,
    ),
    message:
      source.resourceLogs[0].scopeLogs[0].logRecords[0].body.value.stringValue,
    severityNumber:
      source.resourceLogs[0].scopeLogs[0].logRecords[0].severityNumber,
    severityLevel:
      source.resourceLogs[0].scopeLogs[0].logRecords[0].severityText,
    source:
      source.resourceLogs[0].scopeLogs[0].logRecords[0].attributes[0].value
        .stringValue,
    sessionId: source.resourceLogs[0].resource.attributes[0].key,
    serviceName: source.resourceLogs[0].resource.attributes[1].key,
  };
}
