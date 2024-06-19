import { ConsumptionUtil } from "@514labs/moose-lib";

export interface QueryParams {
  step: string;
  from: string;
  to: string;
  hostname: string;
  breakdown: string[];
}
function getDefaultFrom() {
  const oneWeekAgo = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000);
  return Math.floor(oneWeekAgo.getTime() / 1000).toString();
}

function getDefaultTo() {
  const now = new Date();
  return Math.floor(now.getTime() / 1000).toString();
}

export default async function handle(
  {
    step = "3600",
    from = getDefaultFrom(),
    to = getDefaultTo(),
    hostname,
    breakdown,
  }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const stepNum = parseInt(step);

  const hostStub = `%${hostname}%`;
  const hostFilterStub = hostname
    ? sql`AND ${ConsumptionHelpers.column("hostname")} ilike ${hostStub}`
    : sql``;

  return client.query(
    sql`
      SELECT toStartOfInterval(timestamp, interval ${stepNum} second) as timestamp,
      count(*) as hits,
      ${ConsumptionHelpers.column("hostname")},
      FROM PageViewProcessed WHERE timestamp >= fromUnixTimestamp(${parseInt(from)}) AND timestamp < fromUnixTimestamp(${parseInt(to)})
      ${hostFilterStub}
      GROUP BY timestamp, ${ConsumptionHelpers.column("hostname")}
      ORDER BY timestamp ASC WITH FILL FROM toStartOfInterval(fromUnixTimestamp(${parseInt(from)}), interval ${stepNum} second) TO fromUnixTimestamp(${parseInt(to)}) STEP ${stepNum}
  `,
  );
}
