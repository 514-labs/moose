import { ConsumptionUtil } from "@514labs/moose-lib";
interface QueryParams {}

export default async function handle(
  {}: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  return client.query(
    sql`SELECT 
    splitByString('::', source) AS categories,
    count(*) AS occurrences
FROM ParsedLogs_0_5
GROUP BY categories`,
  );
}
