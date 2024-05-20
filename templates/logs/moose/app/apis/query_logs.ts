// Here is a sample api configuration that creates an API which serves the daily active users materialized view

interface QueryParams {
  startTime?: string;
  endTime?: string;
  logSubstring: string;
}

const subtractOneHour = (date: Date) => {
  const dup = new Date(date);
  dup.setHours(dup.getHours() - 1); // out of range is fine
  return dup;
};

export default async function handle(
  { startTime, endTime, logSubstring }: QueryParams,
  { client, sql },
) {
  const end = endTime ? new Date(endTime) : new Date();
  const start = startTime ? new Date(startTime) : subtractOneHour(end);

  console.log("start", start, "end", end);

  const pattern = `%${logSubstring}%`;

  return client.query(
    sql`SELECT * FROM Log
  WHERE coalesce(timestamp, observedTimestamp) >= ${start.getTime() / 1000} AND coalesce(timestamp, observedTimestamp) < ${end.getTime() / 1000} AND
  ilike(body, ${pattern})`,
  );
}
