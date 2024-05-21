export default {
  select: `
SELECT 
count() as bounce_count,
last_page
FROM sessions
WHERE duration > 0
GROUP BY last_page
ORDER BY bounce_count DESC`,
  orderBy: "bounce_count",
};
