import { ConsumptionUtil } from "@514labs/moose-lib";

interface QueryParams {}

export default async function handle(
  {}: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  return client.query(sql`SELECT distinct machineId FROM ParsedLogs_0_5`);
}
