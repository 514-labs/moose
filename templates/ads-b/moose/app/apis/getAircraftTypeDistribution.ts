import {
  createConsumptionApi,
  ConsumptionHelpers as CH,
} from "@514labs/moose-lib";
import { tags } from "typia";

interface QueryParams {
  timeWindow?: number & tags.Type<"int32"> & tags.Minimum<1>;
  minCount?: number & tags.Type<"int32"> & tags.Minimum<1>;
  limit?: number & tags.Type<"int32"> & tags.Minimum<1> & tags.Maximum<100>;
}

export default createConsumptionApi<QueryParams>(
  async ({ timeWindow = 60, minCount = 1, limit = 10 }, { client, sql }) => {
    const query = sql`
      SELECT
        aircraft_type,
        COUNT(*) AS count
      FROM FlattenedAircraftTrackingData_0_0
      WHERE event_time >= now() - INTERVAL ${timeWindow} MINUTE
      GROUP BY aircraft_type
      HAVING COUNT(*) >= ${minCount}
      ORDER BY count DESC
      LIMIT ${limit};
    `;

    const data = await client.query<{ aircraft_type: string; count: number }>(
      query,
    );
    return data;
  },
);
