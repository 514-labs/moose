// This file is where you can define your SQL query for aggregating your data
// from other data models you have defined in Moose. For more information on the
// types of aggregate functions you can run on your existing data, consult the
// Clickhouse documentation: https://clickhouse.com/docs/en/sql-reference/aggregate-functions

interface Aggregation {
  select: string;
  orderBy: string;
}

export default {
  select: `SELECT 
  user_journey,
  count() as visits
FROM sessions
GROUP BY user_journey
ORDER BY visits DESC`,
  orderBy: "visits",
} satisfies Aggregation as Aggregation;
