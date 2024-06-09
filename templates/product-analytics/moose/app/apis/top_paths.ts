import { ConsumptionUtil } from "@514labs/moose-lib";

export interface QueryParams {
  step: string;
  from: number;
  to: number;
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
  { step = "3600", from = 1715497586, to = 1715843186 }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const stepNum = parseInt(step);
  return client.query(
    sql`   
select pathname, count(*) as hits
from PageViewProcessed_0_0
group by pathname
order by hits desc
limit 10`,
  );
}
