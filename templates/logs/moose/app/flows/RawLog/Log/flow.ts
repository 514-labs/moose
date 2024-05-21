// Add your models & start the development server to import these types
import { Log } from "../../../datamodels/models.ts";
import { RawLog, AnyValue, KeyValue } from "../../../otel.ts"; // not great

const rawToDate = (raw: undefined | string | number) =>
  raw === undefined ? undefined : new Date(Number(raw) / 1000000);

const simplifyAttributes = (raw: { attributes: KeyValue[] } | undefined) =>
  raw === undefined
    ? undefined
    : simplifyAny({
        kvlistValue: {
          values: raw.attributes,
        },
      });

const simplifyAny = (raw: AnyValue): any => {
  if ("stringValue" in raw) return raw.stringValue;
  if ("boolValue" in raw) return raw.boolValue;
  if ("intValue" in raw) return raw.intValue.toString();
  if ("doubleValue" in raw) return raw.doubleValue.toString();
  if ("arrayValue" in raw) return raw.arrayValue.values.map(simplifyAny);
  if ("kvlistValue" in raw)
    return raw.kvlistValue.values.map(({ key, value }) => ({
      key,
      value: simplifyAny(value),
    })); // or turn it into JSON
  if ("bytesValue" in raw) return raw.bytesValue;
  return null;
};

// The 'run' function transforms RawLog data to Log format.
// For more details on how Moose flows work, see: https://docs.moosejs.com
export default function run(source: RawLog): Log | null {
  return {
    timestamp: rawToDate(source.timeUnixNano),
    observedTimestamp: rawToDate(source.observedTimeUnixNano),
    traceId: source.traceId,
    spanId: source.spanId,
    traceFlags: source.flags || 0,
    severityText: source.severityText,
    severityNumber: source.severityNumber,
    body: JSON.stringify(simplifyAny(source.body)),
    resource: JSON.stringify(simplifyAttributes(source.resource)),
    instrumentationScopeName: source.scope?.name,
    instrumentationScopeVersion: source.scope?.version,
    instrumentationScopeAttributes: JSON.stringify(
      simplifyAttributes(source.scope),
    ),
    attributes: JSON.stringify(source.attributes),
  };
}
