import { DataModelConfig, IngestionFormat } from "@514labs/moose-lib";

export const LogsConfig: DataModelConfig<Logs> = {
  ingestion: {
    format: IngestionFormat.JSON,
  },
  storage: {
    enabled: false,
    order_by_fields: ["resourceLogs"],
  },
};

interface Attribute {
  key: string;
  value: {
    stringValue: string;
  };
}

interface LogRecord {
  timeUnixNano: string;
  observedTimeUnixNano: string;
  severityNumber: number;
  severityText: string;
  body: {
    value: {
      stringValue: string;
    };
  };
  attributes: Attribute[];
  droppedAttributesCount: number;
  flags: number;
  traceId: string;
  spanId: string;
}

interface ScopeLog {
  scope: {
    name: string;
    version: string;
    attributes: Attribute[];
    droppedAttributesCount: number;
  };
  logRecords: LogRecord[];
  schemaUrl: string;
}

interface ResourceLog {
  resource: {
    attributes: Attribute[];
    droppedAttributesCount: number;
  };
  scopeLogs: ScopeLog[];
  schemaUrl: string;
}

export interface Logs {
  resourceLogs: ResourceLog[];
}

export const ParsedLogsConfig: DataModelConfig<ParsedLogs> = {
  ingestion: {
    format: IngestionFormat.JSON,
  },
  storage: {
    enabled: true,
    order_by_fields: ["date"],
  },
};

export interface ParsedLogs {
  date: Date;
  message: string;
  severityNumber: number;
  severityLevel: string;
  source: string;
  sessionId: string;
  serviceName: string;
  machineId: string;
}
