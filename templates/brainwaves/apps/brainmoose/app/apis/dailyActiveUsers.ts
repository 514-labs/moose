// Here is a sample api configuration that creates an API which serves the daily active users materialized view
import { ConsumptionUtil } from "@514labs/moose-lib";

interface QueryParams {
  limit: string;
  minDailyActiveUsers: string;
}

export default async function handle(
  { limit = "10", minDailyActiveUsers = "0" }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  return client.query(
    sql`SELECT 
      date,
      uniqMerge(dailyActiveUsers) as dailyActiveUsers
  FROM DailyActiveUsers
  GROUP BY date 
  HAVING dailyActiveUsers >= ${parseInt(minDailyActiveUsers)}
  ORDER BY date 
  LIMIT ${parseInt(limit)}`,
  );
}
