import { DataModelConfig, IngestionFormat } from "@514labs/moose-lib";

export interface TimeSeriesData {
  timestamp: Date;
  independentVariables: Record<string, number | string>;
  dependentVariables: Record<string, number>;
}

export const TimeSeriesConfig: DataModelConfig<TimeSeriesData> = {
  ingestion: {
    format: IngestionFormat.JSON,
  },
  storage: {
    enabled: true,
    order_by_fields: ["timestamp"],
  },
};
