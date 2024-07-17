import { ParsedLogs, Logs } from "../../../datamodels/logs";
interface Attribute {
  key: string;
  value: {
    stringValue: string;
  };
}

const getFromAttributes = (attribute: Attribute[], key: string) => {
  const [found] = attribute.filter((v) => v.key === key);
  return found?.value.stringValue;
};

export default function run(source: Logs): ParsedLogs[] {
  let returnArray: ParsedLogs[] = [];

  for (const resourceLog of source.resourceLogs) {
    for (const scopeLog of resourceLog.scopeLogs) {
      for (const logRecord of scopeLog.logRecords) {
        returnArray.push({
          date: new Date(Number(logRecord.observedTimeUnixNano) / 1000000),
          message: logRecord.body.value.stringValue,
          severityNumber: logRecord.severityNumber,
          severityLevel: logRecord.severityText,
          source: logRecord.attributes[0].value.stringValue,
          sessionId: getFromAttributes(
            resourceLog.resource.attributes,
            "session_id",
          ),
          serviceName: getFromAttributes(
            resourceLog.resource.attributes,
            "service_name",
          ),
          machineId: getFromAttributes(
            resourceLog.resource.attributes,
            "machine_id",
          ),
        });
      }
    }
  }

  //return returnArray;
  return returnArray.filter((v) => v.severityLevel !== "DEBUG");
}
