import { ConsumptionUtil } from "@514labs/moose-lib";

export interface QueryParams {
  step: string;
  from: number;
  to: number;
}

function getDefaultFrom(): number {
  const oneWeekAgo = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000);
  return Math.floor(oneWeekAgo.getTime() / 1000);
}

function getDefaultTo(): number {
  const now = new Date();
  return Math.floor(now.getTime() / 1000);
}

export default async function handle(
  { step = "3600", from = getDefaultFrom(), to = getDefaultTo() }: QueryParams,
  { client, sql }: ConsumptionUtil,
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
