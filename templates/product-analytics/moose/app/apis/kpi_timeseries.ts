export interface QueryParams {
  step: string;
  from: number;
  to: number;
}

/*
function makeUnix(date: Date): number {
  return Math.floor(date.getTime() / 1000);
}

function makeDateOffsetDays(date: Date, days: number): Date {
  return new Date(date.getTime() + days * 24 * 60 * 60 * 1000);
}
*/

export default async function handle(
  { step = "3600", from = 1715497586, to = 1715843186 }: QueryParams,
  { client, sql },
) {
  const stepNum = parseInt(step);
  return client.query(
    sql`
    SELECT toStartOfInterval(minute, interval ${stepNum} second) as timestamp,
    sum(visits) as visits,
    avg(avg_session_length) as avg_session_sec,
    sum(total_hits) as pageviews,
    avg(bounce_rate) as bounce_rate
    FROM sessions_over_time WHERE timestamp >= fromUnixTimestamp(${from}) AND timestamp < fromUnixTimestamp(${to})
    GROUP BY timestamp
    ORDER BY timestamp ASC WITH FILL FROM fromUnixTimestamp(${from}) TO fromUnixTimestamp(${to}) STEP ${stepNum}
`,
  );
}
