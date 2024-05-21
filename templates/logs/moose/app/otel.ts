export interface KeyValue {
  key: string;
  value: AnyValue;
}

export type AnyValue =
  | { stringValue: string }
  | { "boolValue:": boolean }
  | { intValue: string | number }
  | { doubleValue: number }
  | { arrayValue: { values: AnyValue[] } }
  | { kvlistValue: { values: KeyValue[] } }
  | { bytesValue: string } // hex probably
  | {};

interface Resource {
  attributes: KeyValue[];
}
interface InstrumentationScope {
  name: string;
  version: string;
  attributes: KeyValue[];
}

export interface RawLog {
  resource?: Resource;
  scope?: InstrumentationScope;
  timeUnixNano?: string | number;
  observedTimeUnixNano: string | number;
  severityNumber?: number;
  severityText?: string;
  traceId?: string; // hex
  spanId?: string; // hex
  body?: AnyValue;
  flags?: number;
  attributes?: AnyValue;
}
