import { ConsumptionUtil, ConsumptionHelpers } from "@514labs/moose-lib";

export interface QueryParams {
  from: string;
  to: string;
  hostname: string;
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
  { from = getDefaultFrom(), to = getDefaultTo(), hostname }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const hostStub = `%${hostname}%`;
  const hostFilterStub = hostname
    ? sql`AND ${ConsumptionHelpers.column("hostname")} ilike ${hostStub}`
    : sql``;

  return client.query(
    sql`
      SELECT timestamp,
      count(*) as hits,
      ${ConsumptionHelpers.column("hostname")},
      FROM PageViewProcessed WHERE timestamp >= fromUnixTimestamp(${parseInt(from)}) AND timestamp < fromUnixTimestamp(${parseInt(to)})
      ${hostFilterStub}
      GROUP BY timestamp, ${ConsumptionHelpers.column("hostname")}
      ORDER BY timestamp ASC
  `,
  );
}
