import { FlattenedAircraftTrackingData } from "../datamodels/flattened";
import { Key } from "@514labs/moose-lib";
import { AircraftTrackingData_altBaroInt } from "datamodels/models";

/**
 * Interleaves bits of two 32-bit numbers to create a Z-order curve value
 * @param lat Latitude value normalized to 32-bit integer
 * @param lon Longitude value normalized to 32-bit integer
 * @returns Z-order curve value
 */
function calculateZOrder(lat: number, lon: number): number {
  // Normalize lat/lon to integers between 0 and 2^20
  const latInt = Math.floor(((lat + 90.0) * (1 << 20)) / 180.0);
  const lonInt = Math.floor(((lon + 180.0) * (1 << 20)) / 360.0);

  // Interleave bits
  let result = 0;
  for (let i = 0; i < 20; i++) {
    result |= ((latInt & (1 << i)) << i) | ((lonInt & (1 << i)) << (i + 1));
  }
  return result;
}

/**
 * Converts NavModes array to boolean flags
 * @param navModes Array of navigation modes
 * @returns Object containing boolean flags for each nav mode
 */
function parseNavModes(navModes?: string[]): {
  approach: boolean;
  autopilot: boolean;
  althold: boolean;
  lnav: boolean;
  tcas: boolean;
} {
  return {
    approach: navModes?.includes("approach") ?? false,
    autopilot: navModes?.includes("autopilot") ?? false,
    althold: navModes?.includes("althold") ?? false,
    lnav: navModes?.includes("lnav") ?? false,
    tcas: navModes?.includes("tcas") ?? false,
  };
}

export default function run(
  source: AircraftTrackingData_altBaroInt,
): FlattenedAircraftTrackingData[] {
  // Calculate Z-order coordinate from lat/lon
  const zorderCoordinate = calculateZOrder(source.lat, source.lon);

  // Parse nav modes into boolean flags
  const navModes = parseNavModes(source.nav_modes);

  const flattened: FlattenedAircraftTrackingData = {
    hex: source.hex,
    type: source.type,
    flight: source.flight,
    r: source.r,
    t: source.t,
    dbFlags: source.dbFlags,
    lat: source.lat,
    lon: source.lon,
    zorderCoordinate,
    timestamp: Date.now(), // Current timestamp in milliseconds
    altBaro: source.alt_baro,
    altBaroIsGround: false, // Set to true if alt_baro is "ground"
    altGeom: source.alt_geom,
    gs: source.gs,
    track: source.track,
    baroRate: source.baro_rate,
    geomRate: source.geom_rate ?? 0,
    squawk: source.squawk,
    emergency: source.emergency,
    category: source.category,
    navQnh: source.nav_qnh ?? 0,
    navAltitudeMcp: source.nav_altitude_mcp ?? 0,
    navHeading: source.nav_heading ?? 0,
    navModesApproach: navModes.approach,
    navModesAutopilot: navModes.autopilot,
    navModesAlthold: navModes.althold,
    navModesLnav: navModes.lnav,
    navModesTcas: navModes.tcas,
    nic: source.nic,
    rc: source.rc,
    seenPos: source.seen_pos,
    version: source.version,
    nicBaro: source.nic_baro,
    nacP: source.nac_p,
    nacV: source.nac_v,
    sil: source.sil,
    silType: source.sil_type,
    gva: source.gva,
    sda: source.sda,
    alert: source.alert,
    spi: source.spi,
    messages: source.messages,
    seen: source.seen,
    rssi: source.rssi,
  };

  return [flattened];
}
