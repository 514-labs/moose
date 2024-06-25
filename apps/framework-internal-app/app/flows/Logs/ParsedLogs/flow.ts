import { ParsedLogs, Logs } from "../../../datamodels/logs";

export default function run(source: Logs): ParsedLogs[] {
  let returnArray: ParsedLogs[] = [];

  //   for (let a = 0; a < source.resourceLogs.length; a++) {
  //     for (let b = 0; b < source.resourceLogs[a].scopeLogs.length; b++) {
  //       for (
  //         let c = 0;
  //         c < source.resourceLogs[a].scopeLogs[b].logRecords.length;
  //         c++
  //       ) {
  //         let d = new Date(
  //           Number(
  //             source.resourceLogs[a].scopeLogs[b].logRecords[c]
  //               .observedTimeUnixNano,
  //           ) / 1000000,
  //         );
  //         returnArray.push({
  //           date: d.toLocaleString(),
  //           message:
  //             source.resourceLogs[a].scopeLogs[b].logRecords[c].body.value
  //               .stringValue,
  //           severityNumber:
  //             source.resourceLogs[a].scopeLogs[b].logRecords[c].severityNumber,
  //           severityLevel:
  //             source.resourceLogs[a].scopeLogs[b].logRecords[c].severityText,
  //           source:
  //             source.resourceLogs[a].scopeLogs[b].logRecords[c].attributes[0]
  //               .value.stringValue,
  //           sessionId: source.resourceLogs[a].resource.attributes[0].key,
  //           serviceName: source.resourceLogs[a].resource.attributes[1].key,
  //         });
  //       }
  //     }
  //   }

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
