export default {
  select: `
      SELECT 
        toStartOfHour(minute) as hour,
        *
      FROM sessions
      GROUP BY hour
      ORDER BY hour ASC`,
  orderBy: "hour",
};
