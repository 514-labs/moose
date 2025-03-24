import { Key, DataModelConfig, IngestionFormat } from "@514labs/moose-lib";

export interface FiberPanelConnection {
  id: Key<string>;
  siteLocation: string;
  rackLocation: string;
  panelName: string;
  portNumber: number;
  isConnected: boolean;
  circuitId: string;
  connectionType: string;
}

export const FiberPanelConnectionConfig: DataModelConfig<FiberPanelConnection> =
  {
    ingestion: {
      format: IngestionFormat.JSON_ARRAY, // Enable batch ingestion (JSON array)
    },
  };
