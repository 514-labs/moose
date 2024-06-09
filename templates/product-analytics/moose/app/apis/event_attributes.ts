import { ConsumptionUtil } from "@514labs/moose-lib";

export interface QueryParams {}

export default async function handle(
  _: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  return client.query(
    sql`
SELECT name, type 
FROM system.columns 
WHERE table = 'PageViewProcessed'
  `,
  );
}
