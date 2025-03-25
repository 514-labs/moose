import {
  createConsumptionApi,
  ConsumptionHelpers as CH,
} from "@514labs/moose-lib";
import { tags } from "typia";

// Define expected parameters and their types
interface QueryParams {
  bucketSize?: number &
    tags.Type<"int32"> &
    tags.Minimum<1> &
    tags.Maximum<10000>;
  timeWindow?: number &
    tags.Type<"int32"> &
    tags.Minimum<1> &
    tags.Maximum<1440>;
  minAltitude?: number &
    tags.Type<"int32"> &
    tags.Minimum<0> &
    tags.Maximum<60000>;
  maxAltitude?: number &
    tags.Type<"int32"> &
    tags.Minimum<0> &
    tags.Maximum<60000>;
}

// createConsumptionApi uses compile time code generation to generate a parser for QueryParams
export default createConsumptionApi<QueryParams>(
  async (
    {
      bucketSize = 2000,
      timeWindow = 60,
      minAltitude = 0,
      maxAltitude = 60000,
    },
    { client, sql },
  ) => {
    const query = sql`
      SELECT
        FLOOR(altitude / ${bucketSize}) * ${bucketSize} AS altitude_bucket,
        COUNT(*) AS count
      FROM
        aircraft_positions
      WHERE
        timestamp >= NOW() - INTERVAL ${timeWindow} MINUTE
        AND altitude >= ${minAltitude}
        AND altitude <= ${maxAltitude}
      GROUP BY
        altitude_bucket
      ORDER BY
        altitude_bucket;
    `;

    // Set return type to the expected query result shape
    const data = await client.query<{ altitude_bucket: number; count: number }>(
      query,
    );

    return data;
  },
);
