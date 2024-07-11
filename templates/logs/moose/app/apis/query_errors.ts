interface QueryParams {}

export default async function handle({}: QueryParams, { client, sql }) {
  return client.query(
    sql`SELECT date, countMerge(errorCount) as count FROM dailyErrors GROUP BY date;`,
  );
}
