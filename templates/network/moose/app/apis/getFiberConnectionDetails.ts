import {
  createConsumptionApi,
  ConsumptionHelpers as CH,
} from "@514labs/moose-lib";
import { tags } from "typia";

// Define expected parameters and their types
interface QueryParams {
  connectionType?: string;
  rackLocation?: string;
  panelName?: string;
  limit?: number & tags.Type<"int32"> & tags.Minimum<1> & tags.Maximum<1000>;
}

// createConsumptionApi uses compile time code generation to generate a parser for QueryParams
export default createConsumptionApi<QueryParams>(
  async (
    { connectionType, rackLocation, panelName, limit = 100 },
    { client, sql },
  ) => {
    // Build filter conditions based on parameters
    const conditions = [];

    if (connectionType) {
      conditions.push(sql`connectionType = ${connectionType}`);
    }

    if (rackLocation) {
      conditions.push(sql`rackLocation = ${rackLocation}`);
    }

    if (panelName) {
      conditions.push(sql`panelName = ${panelName}`);
    }

    // Combine conditions with AND
    const whereClause =
      conditions.length > 0
        ? sql`WHERE ${CH.join(conditions, " AND ")}`
        : sql``;

    // Query for detailed connection information
    const connections = await client.query.execute(sql`
      SELECT 
        id,
        siteLocation,
        rackLocation,
        panelName,
        portNumber,
        isConnected,
        circuitId,
        connectionType
      FROM FiberPanelConnection_0_0
      ${whereClause}
      ORDER BY rackLocation, panelName, portNumber
      LIMIT ${limit}
    `);

    // Get circuit ID patterns
    const circuitPatterns = await client.query(sql`
      SELECT 
        substring(circuitId, 1, position(' ', circuitId) - 1) as circuitPrefix,
        COUNT(*) as count
      FROM FiberPanelConnection_0_0
      ${whereClause}
      WHERE circuitId != '' AND position(' ', circuitId) > 0
      GROUP BY circuitPrefix
      ORDER BY count DESC
      LIMIT 10
    `);

    // Get connections per rack location
    const rackCounts = await client.query(sql`
      SELECT 
        rackLocation,
        COUNT(*) as connectionCount
      FROM FiberPanelConnection_0_0
      ${whereClause}
      GROUP BY rackLocation
      ORDER BY connectionCount DESC
    `);

    return {
      totalConnections: connections.length,
      connections,
      circuitPatterns,
      rackCounts,
      appliedFilters: {
        connectionType: connectionType || null,
        rackLocation: rackLocation || null,
        panelName: panelName || null,
        limit,
      },
    };
  },
);
