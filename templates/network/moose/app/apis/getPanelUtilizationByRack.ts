import {
  createConsumptionApi,
  ConsumptionHelpers as CH,
} from "@514labs/moose-lib";
import { tags } from "typia";

// Define expected parameters and their types
interface QueryParams {
  minUtilization?: number & tags.Minimum<1> & tags.Maximum<100>;
  maxUtilization?: number & tags.Minimum<1> & tags.Maximum<100>;
}

// createConsumptionApi uses compile time code generation to generate a parser for QueryParams
export default createConsumptionApi<QueryParams>(
  async ({ minUtilization = 0, maxUtilization = 100 }, { client, sql }) => {
    const query = sql`
      SELECT
        rackLocation as rack_location,
        AVG(100.0 * connectedPorts / totalPorts) AS avg_utilization
      FROM
        FiberPanel_0_0
      WHERE
        (100.0 * connectedPorts / totalPorts) >= ${minUtilization}
        AND (100.0 * connectedPorts / totalPorts) <= ${maxUtilization}
      GROUP BY
        rackLocation
      ORDER BY
        avg_utilization DESC
    `;

    // Set return type to the expected query result shape
    const data = await client.query.execute<{
      rack_location: string;
      avg_utilization: number;
    }>(query);

    return data;
  },
);
