import { ConsumptionUtil } from "@514labs/moose-lib";
import { PageViewProcessed } from "../../.moose/moose-sdk/index";

// this will eventually take a table name, but for now it's hardcoded
interface FilterOperator {
  input: string;
  operator: string;
  condition: string;
}
/*
Group {
	columns: [key, value, type}
	filters: [rejection policies]
	values : [ Group or table(s?)]
}
*/

interface Group {
  groupName: string;
  groups: Group[] | null;
  filters: FilterOperator[];
}

interface QueryConfig {
  // Column names to group by
}

function queryBuilder() {}
export default async function handle(
  { limit = "100", hostname = "moosejs", step = "60" }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const limitNum = parseInt(limit);
  const stepNum = parseInt(step);

  return client.query(
    sql`
    SELECT toStartOfInterval(timestamp, interval ${stepNum} second) as timestep,
         count(distinct session_id) as total_sessions FROM PageViewProcessed_0_0
        WHERE position(hostname, ${hostname}) > 0
        GROUP BY timestep
        ORDER BY timestep ASC WITH FILL STEP ${stepNum}
         LIMIT ${limitNum};
    `,
  );
}
