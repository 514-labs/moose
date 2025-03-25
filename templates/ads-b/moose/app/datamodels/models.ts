import { Key, DataModelConfig } from "@514labs/moose-lib";

interface NavModes {
  approach?: boolean;
  autopilot?: boolean;
  althold?: boolean;
  lnav?: boolean;
  tcas?: boolean;
}

export interface AircraftTrackingData_altBaroInt {
  // Aircraft identifiers
  hex: Key<string>; // using hex as the key since it appears to be a unique aircraft identifier
  type: string;
  flight: string;
  r: string;
  t: string;
  dbFlags: number;

  // Position data
  lat: number;
  lon: number;
  alt_baro: number;
  alt_geom: number;
  gs: number;
  track: number;
  baro_rate: number;
  geom_rate?: number;
  squawk: string;

  // Status information
  emergency: string;
  category: string;
  nav_qnh?: number;
  nav_altitude_mcp?: number;
  nav_heading?: number;
  nav_modes?: string[];

  // Technical parameters
  nic: number;
  rc: number;
  seen_pos: number;
  version: number;
  nic_baro: number;
  nac_p: number;
  nac_v: number;
  sil: number;
  sil_type: string;
  gva: number;
  sda: number;

  // Status flags
  alert: number;
  spi: number;

  // Arrays
  mlat: string[];
  tisb: string[];

  // Additional metrics
  messages: number;
  seen: number;
  rssi: number;
}

export const AircraftTrackingData_altBaroIntConfig: DataModelConfig<AircraftTrackingData_altBaroInt> =
  {
    storage: {
      enabled: false,
    },
  };
