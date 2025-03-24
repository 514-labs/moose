import { Key, DataModelConfig, IngestionFormat } from "@514labs/moose-lib";

export interface TransportSchedule {
  id: Key<string>;
  transportRingName: string;
  priority: number;
  ispReadyDate: Date;
  fieldOpsDate: Date;
  hubARegion: string;
  hubAMarket: string;
  hubALocation: string;
  hubAName: string;
  hubACLLI: string;
  hubACity: string;
  hubAState: string;
  hubBRegion: string;
  hubBMarket: string;
  hubBLocation: string;
  hubBName: string;
  hubBCLLI: string;
  hubBCity: string;
  hubBState: string;
  distanceKM: number;
  hubAToHubB: string;
  hubACLLIToHubBCLLI: string;
  comments: string;
  ispActualDateComplete: string;
}

export const TransportScheduleConfig: DataModelConfig<TransportSchedule> = {
  ingestion: {
    format: IngestionFormat.JSON_ARRAY, // Enable batch ingestion (JSON array)
  },
};
