// Here is a sample aggregation query that calculates the number of daily active users
// based on the number of unique users who complete a sign-in activity each day.

import {
  createAggregation,
  dropAggregation,
  Blocks,
  ClickHouseEngines,
} from "@514labs/moose-lib";

const DESTINATION_TABLE = "DailyActiveUsers";
const MATERIALIZED_VIEW = "DailyActiveUsers_mv";
const SELECT_QUERY = `
SELECT 
    toStartOfDay(timestamp) as date,
    uniqState(userId) as dailyActiveUsers
FROM ParsedActivity_0_0 
WHERE activity = 'Login' 
GROUP BY toStartOfDay(timestamp) 
`;

export default {
  teardown: [
    ...dropAggregation({
      viewName: MATERIALIZED_VIEW,
      tableName: DESTINATION_TABLE,
    }),
  ],
  setup: [
    ...createAggregation({
      tableCreateOptions: {
        name: DESTINATION_TABLE,
        columns: {
          date: "Date",
          dailyActiveUsers: "AggregateFunction(uniq, String)",
        },
        engine: ClickHouseEngines.AggregatingMergeTree,
        orderBy: "date",
      },
      materializedViewName: MATERIALIZED_VIEW,
      select: SELECT_QUERY,
    }),
  ],
} as Blocks;
