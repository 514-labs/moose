type Key<T extends string | number> = T;

export interface RawLog {}

export const RawLogConfig = {
  storage: {
    enabled: false,
  },
};

export interface Log {
  timestamp?: Date;
  observedTimestamp: Date;
  traceId?: string;
  spanId?: string;
  traceFlags?: number;
  severityText?: string; // INFO/ERROR...
  severityNumber?: number; // 1-24
  body?: string; // JSON representation of https://opentelemetry.io/docs/specs/otel/logs/data-model/#type-any
  resource?: string; // JSON
  instrumentationScopeName?: string;
  instrumentationScopeVersion?: string;
  instrumentationScopeAttributes?: string; // JSON
  attributes?: string; // JSON
}

export const LogConfig = {
  storage: {
    order_by_fields: ["observedTimestamp"],
  },
};
