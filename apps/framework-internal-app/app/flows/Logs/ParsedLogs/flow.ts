import { ParsedLogs, Logs } from "../../../datamodels/logs";

export default function run(source: Logs): ParsedLogs[] {
  let returnArray: ParsedLogs[] = [];

  for (const resourceLog of source.resourceLogs) {
    for (const scopeLog of resourceLog.scopeLogs) {
      for (const logRecord of scopeLog.logRecords) {
        let d = new Date(Number(logRecord.observedTimeUnixNano) / 1000000);
        returnArray.push({
          date: d.toLocaleString(),
          message: logRecord.body.value.stringValue,
          severityNumber: logRecord.severityNumber,
          severityLevel: logRecord.severityText,
          source: logRecord.attributes[0].value.stringValue,
          sessionId: resourceLog.resource.attributes[0].key,
          serviceName: resourceLog.resource.attributes[1].key,
        });
      }
    }
  }

  return returnArray;
}
