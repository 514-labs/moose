export interface QueryParams {
  step: string;
  from: number;
  to: number;
  hostname: string;
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
  { step = "3600", from = 1715497586, to = 1715843186, hostname }: QueryParams,
  { client, sql, helpers },
) {
  const stepNum = parseInt(step);

  const hostStub = `%${hostname}%`;
  const hostFilterStub = hostname
    ? sql`AND ${helpers.column("hostname")} ilike ${hostStub}`
    : sql``;

  return client.query(
    sql`
      SELECT toStartOfInterval(timestamp, interval ${stepNum} second) as timestamp,
      count(*) as hits,
      ${helpers.column("hostname")},
      FROM PageViewProcessed WHERE timestamp >= fromUnixTimestamp(${from}) AND timestamp < fromUnixTimestamp(${to})
      ${hostFilterStub}
      GROUP BY timestamp, ${helpers.column("hostname")}
      ORDER BY timestamp ASC WITH FILL FROM toStartOfInterval(fromUnixTimestamp(${from}), interval ${stepNum} second) TO fromUnixTimestamp(${to}) STEP ${stepNum}
  `,
  );
}
