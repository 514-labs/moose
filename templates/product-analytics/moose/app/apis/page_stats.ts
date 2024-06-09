import { ConsumptionUtil } from "@514labs/moose-lib";

export interface QueryParams {
  limit: string;
  hostname: string;
  step: string;
}

export default async function handle(
  { limit = "100", hostname = "moosejs", step = "60" }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const limitNum = parseInt(limit);
  const stepNum = parseInt(step);

  return client.query(
    sql`
    SELECT toStartOfInterval(timestamp, interval ${stepNum} second) as timestep,
         count(distinct session_id) as total_sessions FROM PageViewProcessed_0_0
        WHERE position(hostname, ${hostname}) > 0
        GROUP BY timestep
        ORDER BY timestep ASC WITH FILL STEP ${stepNum}
         LIMIT ${limitNum};
    `,
  );
}
