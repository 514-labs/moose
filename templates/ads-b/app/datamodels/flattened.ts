import { Key, DataModelConfig } from "@514labs/moose-lib";

export interface FlattenedAircraftTrackingData {
  hex: Key<string>;
  type: string;
  flight: string;
  r: string;
  t: string;
  dbFlags: number;
  lat: number;
  lon: number;
  zorderCoordinate: number;
  timestamp: number;
  altBaro: number;
  altBaroIsGround: boolean;
  altGeom: number;
  gs: number;
  track: number;
  baroRate: number;
  geomRate: number;
  squawk: string;
  emergency: string;
  category: string;
  navQnh: number;
  navAltitudeMcp: number;
  navHeading: number;
  navModesApproach: boolean;
  navModesAutopilot: boolean;
  navModesAlthold: boolean;
  navModesLnav: boolean;
  navModesTcas: boolean;
  nic: number;
  rc: number;
  seenPos: number;
  version: number;
  nicBaro: number;
  nacP: number;
  nacV: number;
  sil: number;
  silType: string;
  gva: number;
  sda: number;
  alert: number;
  spi: number;
  messages: number;
  seen: number;
  rssi: number;
}

// export const FlattenedAircraftTrackingDataConfig: DataModelConfig<FlattenedAircraftTrackingData> = {
//   storage: {
//     enabled: true,
//     order_by_fields: ["zorderCoordinate"] // optimize queries by creation date
//   }
// };
