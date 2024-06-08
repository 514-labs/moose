from dataclasses import dataclass

@dataclass
class Aggregation:
    select: str
    orderBy: str

sql = """
SELECT 
    uniqState(userId) as dailyActiveUsers,
    toStartOfDay(timestamp) as date
FROM ParsedActivity_0_0
WHERE activity = 'Login' 
GROUP BY toStartOfDay(timestamp)
"""

agg = Aggregation(select=sql,orderBy="date")