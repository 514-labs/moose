import { ConsumptionApi } from "@514labs/moose-lib";
import { tags } from "typia";

/**
 * Parameters for the aircraft positions API
 */
interface AircraftPositionsParams {
  /** Maximum number of aircraft to return (1-1000) */
  limit?: number & tags.Type<"int64"> & tags.Minimum<1> & tags.Maximum<1000>;

  /** Minimum altitude in feet to filter by (optional) */
  minAltitude?: number & tags.Type<"int64"> & tags.Minimum<0>;

  /** Maximum altitude in feet to filter by (optional) */
  maxAltitude?: number & tags.Type<"int64"> & tags.Minimum<0>;
}

/**
 * API to retrieve current positions and details of all active aircraft
 * Returns aircraft locations, altitude, speed, and identifying information
 */
export const getAircraftPositions = new ConsumptionApi<AircraftPositionsParams>(
  "getAircraftPositions",
  async (params, utils) => {
    const { client, sql } = utils;

    // Set default limit if not provided
    const limit = params.limit ?? 100;

    // Build the query with conditional altitude filters
    let query = sql`
      SELECT 
        hex,
        flight,
        aircraft_type,
        category,
        lat,
        lon,
        alt_baro as altitude,
        gs as ground_speed,
        track as heading,
        emergency,
        alt_baro_is_ground as is_on_ground,
        timestamp
      FROM AircraftTrackingProcessed
      WHERE lat != 0 AND lon != 0  -- Filter out invalid positions
        AND timestamp >= now() - INTERVAL 30 SECOND  -- Only show recent aircraft
    `;

    // Add minimum altitude filter if provided
    if (params.minAltitude !== undefined) {
      query = sql`${query} AND alt_baro >= ${params.minAltitude}`;
    }

    // Add maximum altitude filter if provided
    if (params.maxAltitude !== undefined) {
      query = sql`${query} AND alt_baro <= ${params.maxAltitude}`;
    }

    // Add ordering and limit
    query = sql`
      ${query}
      ORDER BY timestamp DESC
      LIMIT ${limit}
    `;

    // Execute the query and return results
    return await client.query.execute(query);
  },
);
