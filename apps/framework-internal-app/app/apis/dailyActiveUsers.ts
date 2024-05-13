interface QueryParams {
  limit: string;
  minDailyActiveUsers: string;
}

export default async function handle(
  { limit = "10", minDailyActiveUsers = "0" }: QueryParams,
  { client, sql },
) {
  return client.query(
    sql`SELECT 
        date,
        dailyActiveUsers
    FROM DailyActiveUsers_aggregations_mv
    WHERE dailyActiveUsers >= ${parseInt(minDailyActiveUsers)}
    LIMIT ${parseInt(limit)}`,
  );
}
