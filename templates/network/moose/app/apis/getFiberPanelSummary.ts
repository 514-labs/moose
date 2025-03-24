import {
  createConsumptionApi,
  ConsumptionHelpers as CH,
} from "@514labs/moose-lib";
import { tags } from "typia";

// Define expected parameters and their types
interface QueryParams {
  siteLocation?: string;
}

// createConsumptionApi uses compile time code generation to generate a parser for QueryParams
export default createConsumptionApi<QueryParams>(
  async ({ siteLocation }, { client, sql }) => {
    // Build the site location filter if provided
    const siteLocationFilter = siteLocation
      ? sql`AND siteLocation = ${siteLocation}`
      : sql``;

    // Get overall summary
    const overallSummary = await client.query.execute(sql`
      SELECT 
        COUNT(*) as panelCount,
        SUM(totalPorts) as totalPorts,
        SUM(connectedPorts) as connectedPorts,
        ROUND(SUM(connectedPorts) / SUM(totalPorts) * 100, 2) as utilizationRate
      FROM FiberPanel_0_0
      WHERE 1=1 ${siteLocationFilter}
    `);

    // Get panel details
    const panelDetails = await client.query.execute(sql`
      SELECT 
        rackLocation,
        panelName,
        connectedPorts,
        totalPorts,
        ROUND(connectedPorts / totalPorts * 100, 2) as utilizationRate
      FROM FiberPanel_0_0
      WHERE 1=1 ${siteLocationFilter}
      ORDER BY utilizationRate DESC
    `);

    // Get connection types summary
    const connectionTypes = await client.query.execute(sql`
      SELECT 
        IF(connectionType = '', 'Unspecified', connectionType) as connectionType, 
        COUNT(*) as connectionCount,
        ROUND(COUNT(*) / (
          SELECT COUNT(*) 
          FROM FiberPanelConnection_0_0 
          WHERE 1=1 ${siteLocationFilter}
        ) * 100, 2) as percentage
      FROM FiberPanelConnection_0_0
      WHERE 1=1 ${siteLocationFilter}
      GROUP BY connectionType
      ORDER BY connectionCount DESC
    `);

    // Get top panels by connection count
    const topPanels = await client.query.execute(sql`
      SELECT 
        rackLocation,
        panelName,
        COUNT(*) as connectionCount
      FROM FiberPanelConnection_0_0
      WHERE 1=1 ${siteLocationFilter}
      GROUP BY rackLocation, panelName
      ORDER BY connectionCount DESC
      LIMIT 5
    `);

    return {
      summary: overallSummary[0] || {
        panelCount: 0,
        totalPorts: 0,
        connectedPorts: 0,
        utilizationRate: 0,
      },
      panels: panelDetails,
      connectionTypes,
      topPanels,
    };
  },
);
