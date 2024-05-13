// Here is a sample aggregation query that calculates the number of daily active users
// based on the number of unique users who complete a sign-in activity each day.

interface Aggregation {
  select: string;
  orderBy: string;
}

export default {
  select: ` 
    SELECT 
        count(distinct userId) as dailyActiveUsers,
        toStartOfDay(timestamp) as date
    FROM ParsedActivity_0_3
    WHERE activity = 'Login' 
    GROUP BY toStartOfDay(timestamp)
    `,
  orderBy: "date",
} satisfies Aggregation as Aggregation;
