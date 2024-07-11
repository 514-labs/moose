import { ConsumptionUtil } from "@514labs/moose-lib";

export default async function handle(
  _unused: any,
  { client, sql }: ConsumptionUtil,
) {
  return client.query(
    sql`
    SELECT 
    name AS metricName
FROM system.tables
WHERE database == 'local'
AND engine == 'View'
`,
  );
}
