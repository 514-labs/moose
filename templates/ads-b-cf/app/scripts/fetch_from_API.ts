import { ETLPipeline } from "@514labs/moose-lib";
import { APIContext, APISource } from "@514labs/moose-connector-api";
import { AircraftTrackingData, LADDAircraftData } from "datamodels/models";

// =============================================================================
// API RESPONSE INTERFACES
// =============================================================================

// Standard military aircraft API response
interface ApiResponse {
  ac: AircraftTrackingData[];
  total?: number;
  ctime?: number;
  ptime?: number;
  msg?: string;
  now?: number;
}

// LADD (Limiting Aircraft Data Displayed) API response
interface LADDApiResponse {
  ac: LADDApiAircraftData[];
  ctime?: number;
  msg?: string;
  now?: number;
  ptime?: number;
  total?: number;
}

// Raw LADD API aircraft data with original field names from API
interface LADDApiAircraftData extends AircraftTrackingData {
  t?: string;
  type?: string;
  true_heading?: number;
  ias?: number;
  mach?: number;
  mag_heading?: number;
  oat?: number;
  roll?: number;
  tas?: number;
  tat?: number;
  track_rate?: number;
  wd?: number;
  ws?: number;
  gpsOkBefore?: number;
  gpsOkLat?: number;
  gpsOkLon?: number;
  lastPosition?: {
    lat: number;
    lon: number;
    nic: number;
    rc: number;
    seen_pos: number;
  };
  rr_lat?: number;
  rr_lon?: number;
  calc_track?: number;
  nav_altitude_fms?: number;
}

// =============================================================================
// API SOURCE CONNECTORS
// =============================================================================

/**
 * APISource for military aircraft data
 */
const militaryAircraftAPISource = new APISource<
  ApiResponse,
  AircraftTrackingData[]
>({
  name: "military_aircraft_api",
  baseUrl: "https://api.adsb.lol",
  endpoint: "/v2/mil",
  headers: {
    "User-Agent": "Moose-Aircraft-Tracker/1.0",
  },
  pagination: {
    getNextUrl: () => null,
    maxPages: 1,
    delayBetweenRequests: 0,
    retryConfig: {
      maxRetries: 5,
      backoffMs: 1000,
    },
  },
  extractItems: (context: APIContext<ApiResponse>) => {
    const response = context.response;

    if (response.total) {
      console.log(`Military API returned ${response.total} aircraft records`);
    }

    return response.ac || [];
  },
});

/**
 * APISource for LADD aircraft data
 */
const laddAircraftAPISource = new APISource<
  LADDApiResponse,
  LADDApiAircraftData[]
>({
  name: "ladd_aircraft_api",
  baseUrl: "https://api.adsb.lol",
  endpoint: "/v2/ladd",
  headers: {
    "User-Agent": "Moose-Aircraft-Tracker/1.0",
  },
  pagination: {
    getNextUrl: () => null,
    maxPages: 1,
    delayBetweenRequests: 0,
    retryConfig: {
      maxRetries: 5,
      backoffMs: 1500,
    },
  },
  extractItems: (context: APIContext<LADDApiResponse>) => {
    const response = context.response;

    if (response.total) {
      console.log(`LADD API returned ${response.total} aircraft records`);
    }
    if (response.msg) {
      console.log(`LADD API message: ${response.msg}`);
    }

    return response.ac || [];
  },
});

// =============================================================================
// TRANSFORM FUNCTIONS
// =============================================================================

/**
 * Transform military aircraft data to standardized format
 * Handles adsb.lol API quirk where alt_baro can be "ground" string
 */
async function transformAircraftData(
  aircraft: AircraftTrackingData,
): Promise<AircraftTrackingData> {
  let alt_baro = 0;
  let alt_baro_is_ground = false;

  if (typeof aircraft.alt_baro === "string" && aircraft.alt_baro === "ground") {
    alt_baro = 0;
    alt_baro_is_ground = true;
  } else if (typeof aircraft.alt_baro === "number" && aircraft.alt_baro === 0) {
    alt_baro_is_ground = true;
  } else {
    alt_baro = aircraft.alt_baro || 0;
  }

  return {
    ...aircraft,
    source: "Mil",
    alt_baro,
    alt_baro_is_ground,
    timestamp: new Date(),
    // Ensure all required fields have defaults
    transponder_type: aircraft.transponder_type || "",
    flight: aircraft.flight || "",
    r: aircraft.r || "",
    aircraft_type: aircraft.aircraft_type || "",
    dbFlags: aircraft.dbFlags || 0,
    lat: aircraft.lat || 0,
    lon: aircraft.lon || 0,
    alt_geom: aircraft.alt_geom || 0,
    gs: aircraft.gs || 0,
    track: aircraft.track || 0,
    baro_rate: aircraft.baro_rate || 0,
    squawk: aircraft.squawk || "",
    emergency: aircraft.emergency || "",
    category: aircraft.category || "",
    nic: aircraft.nic || 0,
    rc: aircraft.rc || 0,
    seen_pos: aircraft.seen_pos || 0,
    version: aircraft.version || 0,
    nic_baro: aircraft.nic_baro || 0,
    nac_p: aircraft.nac_p || 0,
    nac_v: aircraft.nac_v || 0,
    sil: aircraft.sil || 0,
    sil_type: aircraft.sil_type || "",
    gva: aircraft.gva || 0,
    sda: aircraft.sda || 0,
    alert: aircraft.alert || 0,
    spi: aircraft.spi || 0,
    mlat: aircraft.mlat || [],
    tisb: aircraft.tisb || [],
    messages: aircraft.messages || 0,
    seen: aircraft.seen || 0,
    rssi: aircraft.rssi || 0,
  };
}

/**
 * Transform LADD aircraft data to enriched format
 * Maps raw API field names to standardized field names
 */
async function transformLADDAircraftData(
  aircraft: LADDApiAircraftData,
): Promise<LADDAircraftData> {
  // Use base transformation for common fields
  const baseData = await transformAircraftData(aircraft);

  // Add LADD-specific fields with field name mapping
  return {
    ...baseData,
    aircraft_type: aircraft.type || aircraft.aircraft_type || "",
    source: "LADD",
    true_heading: aircraft.true_heading || 0,
    indicated_airspeed: aircraft.ias || 0,
    mach_number: aircraft.mach || 0,
    magnetic_heading: aircraft.mag_heading || 0,
    outside_air_temp: aircraft.oat || 0,
    roll_angle: aircraft.roll || 0,
    true_airspeed: aircraft.tas || 0,
    total_air_temp: aircraft.tat || 0,
    track_rate: aircraft.track_rate || 0,
    wind_direction: aircraft.wd || 0,
    wind_speed: aircraft.ws || 0,
    gps_ok_before: aircraft.gpsOkBefore || 0,
    gps_ok_lat: aircraft.gpsOkLat || 0,
    gps_ok_lon: aircraft.gpsOkLon || 0,
    last_position_lat: aircraft.lastPosition?.lat || 0,
    last_position_lon: aircraft.lastPosition?.lon || 0,
    last_position_nic: aircraft.lastPosition?.nic || 0,
    receiver_lat: aircraft.rr_lat || 0,
    receiver_lon: aircraft.rr_lon || 0,
    calculated_track: aircraft.calc_track || 0,
    nav_altitude_fms: aircraft.nav_altitude_fms || 0,
  };
}

// =============================================================================
// LOAD FUNCTIONS
// =============================================================================

/**
 * Batch load military aircraft data to ingestion endpoint
 */
async function loadAircraftDataBatch(
  transformedDataArray: AircraftTrackingData[],
): Promise<void> {
  let processedCount = 0;

  for (const transformedData of transformedDataArray) {
    try {
      const response = await fetch(
        "http://localhost:4000/ingest/AircraftTrackingData",
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(transformedData),
        },
      );

      if (response.ok) {
        processedCount++;
      } else {
        console.warn(
          `Failed to load aircraft ${transformedData.hex}: ${response.status}`,
        );
      }
    } catch (error) {
      console.warn(`Error loading aircraft ${transformedData.hex}:`, error);
    }
  }

  console.log(
    `Batch load completed: ${processedCount}/${transformedDataArray.length} records processed`,
  );
}

/**
 * Batch load LADD aircraft data to dedicated endpoint
 */
async function loadLADDAircraftDataBatch(
  transformedDataArray: LADDAircraftData[],
): Promise<void> {
  let processedCount = 0;

  for (const transformedData of transformedDataArray) {
    try {
      const response = await fetch(
        "http://localhost:4000/ingest/LADDAircraftData",
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(transformedData),
        },
      );

      if (response.ok) {
        processedCount++;
      } else {
        console.warn(
          `Failed to load LADD aircraft ${transformedData.hex}: ${response.status}`,
        );
      }
    } catch (error) {
      console.warn(
        `Error loading LADD aircraft ${transformedData.hex}:`,
        error,
      );
    }
  }

  console.log(
    `LADD batch load completed: ${processedCount}/${transformedDataArray.length} records processed`,
  );
}

// =============================================================================
// ETL PIPELINES
// =============================================================================

/**
 * ETL pipeline for military aircraft data
 * Run with: moose workflow run militaryAircraftETL
 */
export const militaryAircraftETL = new ETLPipeline<
  AircraftTrackingData,
  AircraftTrackingData
>("militaryAircraftETL", {
  extract: militaryAircraftAPISource,
  transform: transformAircraftData,
  load: loadAircraftDataBatch,
});

/**
 * ETL pipeline for LADD aircraft data
 * Run with: moose workflow run laddAircraftETL
 */
export const laddAircraftETL = new ETLPipeline<
  LADDApiAircraftData,
  LADDAircraftData
>("laddAircraftETL", {
  extract: laddAircraftAPISource,
  transform: transformLADDAircraftData,
  load: loadLADDAircraftDataBatch,
});

/**
 * Default export
 */
export default function () {
  return militaryAircraftETL;
}
