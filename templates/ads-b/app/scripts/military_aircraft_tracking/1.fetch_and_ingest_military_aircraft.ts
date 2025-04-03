import { TaskFunction, TaskDefinition } from "@514labs/moose-lib";
import * as http from "http";

// Define interfaces for the API response and task data
interface AircraftData {
  hex: string;
  flight: string;
  alt_baro: number;
  lat: number;
  lon: number;
  track: number;
  speed: number;
  category: string;
  mlat: boolean;
  tisb: boolean;
  messages: number;
  seen: number;
  rssi: number;
}

interface ApiResponse {
  aircraft: AircraftData[];
  total: number;
  time: number;
}

interface ProcessedAircraftData {
  hex: string;
  flight: string;
  altitude_int: number;
  altitude_str: string;
  lat: number;
  lon: number;
  track: number;
  speed: number;
  category: string;
  is_military: boolean;
  timestamp: string;
}

async function mapToAircraftTrackingData(
  aircraft: any,
  timestamp: string,
): Promise<any> {
  // Convert alt_baro to number and handle "ground" case
  let alt_baro = 0;
  let alt_baro_is_ground = false;

  if (aircraft.alt_baro === "ground") {
    alt_baro = 0;
    alt_baro_is_ground = true;
  } else {
    alt_baro = parseInt(aircraft.alt_baro) || 0;
  }

  return {
    hex: aircraft.hex,
    transponder_type: aircraft.type || "",
    transponder_type: aircraft.type || "",
    flight: aircraft.flight || "",
    r: aircraft.r || "",
    aircraft_type: aircraft.t || "",
    aircraft_type: aircraft.t || "",
    dbFlags: aircraft.dbFlags || 0,
    lat: aircraft.lat || 0,
    lon: aircraft.lon || 0,
    alt_baro: alt_baro,
    alt_baro_is_ground: alt_baro_is_ground,
    alt_geom: aircraft.alt_geom || 0,
    gs: aircraft.gs || 0,
    track: aircraft.track || 0,
    baro_rate: aircraft.baro_rate || 0,
    geom_rate: aircraft.geom_rate,
    squawk: aircraft.squawk || "",
    emergency: aircraft.emergency || "",
    category: aircraft.category || "",
    nav_qnh: aircraft.nav_qnh,
    nav_altitude_mcp: aircraft.nav_altitude_mcp,
    nav_heading: aircraft.nav_heading,
    nav_modes: aircraft.nav_modes,
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
    timestamp: timestamp,
    timestamp: timestamp,
  };
}

async function sendToMoose(data: any, endpoint: string): Promise<void> {
  const options = {
    hostname: "localhost",
    port: 4000,
    path: `/ingest/${endpoint}`,
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
  };

  return new Promise((resolve, reject) => {
    const req = http.request(options, (res) => {
      let responseData = "";
      res.on("data", (chunk) => {
        responseData += chunk;
      });
      res.on("end", () => {
        if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) {
          console.log(`Successfully sent to Moose: ${responseData}`);
          resolve();
        } else {
          reject(new Error(`HTTP Error: ${res.statusCode} - ${responseData}`));
        }
      });
    });

    req.on("error", (error) => {
      reject(error);
    });

    req.write(JSON.stringify(data));
    req.end();
  });
}

/**
 * Fetches military aircraft data from adsb.lol API and processes it for Moose ingestion
 *
 * @param input - Input parameters (not used in this task)
 * @returns Object containing task name and processed aircraft data
 */
const fetchAndIngestMilitaryAircraft: TaskFunction = async (input: any) => {
  try {
    console.log("Fetching military aircraft data from adsb.lol API");

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 30000);

    const response = await fetch("https://api.adsb.lol/v2/mil", {
      method: "GET",
      signal: controller.signal,
    });

    clearTimeout(timeoutId);

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();

    // Add collection timestamp
    const timestamp = new Date().toISOString();
    const enrichedData = {
      ...data,
      collectionTimestamp: timestamp,
    };

    console.log(JSON.stringify(enrichedData));

    // Process and send each aircraft to Moose
    if (enrichedData.ac && Array.isArray(enrichedData.ac)) {
      for (const aircraft of enrichedData.ac) {
        try {
          const mappedData = await mapToAircraftTrackingData(
            aircraft,
            timestamp,
          );
          await sendToMoose(mappedData, "AircraftTrackingData");
        } catch (error) {
          console.log(`Error processing aircraft ${aircraft.hex}: ${error}`);
        }
      }
    }

    return {
      task: "fetch_and_ingest_military_aircraft",
      data: enrichedData,
    };
  } catch (error) {
    console.error(
      `Error: ${error instanceof Error ? error.message : String(error)}`,
    );
    throw error;
  }
};

/**
 * Creates and returns the task definition for the Moose workflow
 */
export default function createTask(): TaskDefinition {
  return {
    task: fetchAndIngestMilitaryAircraft,
    config: {
      retries: 3,
    },
  } as TaskDefinition;
}
