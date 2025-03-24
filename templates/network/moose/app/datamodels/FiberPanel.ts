import { Key, DataModelConfig, IngestionFormat } from "@514labs/moose-lib";

export interface FiberPanel {
  id: Key<string>;
  siteLocation: string;
  rackLocation: string;
  panelName: string;
  connectedPorts: number;
  totalPorts: number;
}

export const FiberPanelConfig: DataModelConfig<FiberPanel> = {
  ingestion: {
    format: IngestionFormat.JSON_ARRAY, // Enable batch ingestion (JSON array)
  },
};
