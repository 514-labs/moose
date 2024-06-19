SELECT 
    project,
    activityType,
    groupArray((timestamp, count)) as timeseries,
            sum(count) as total_count

FROM (
    SELECT 
        project,
        activityType,
        toStartOfInterval(timestamp, interval 3600 second) as timestamp,
        count(*) as count
    FROM MooseActivityAugmented
    GROUP BY project, activityType, timestamp
    ORDER BY project, activityType, timestamp ASC
    WITH FILL FROM toStartOfInterval(fromUnixTimestamp(1718139452), interval 3600 second) TO fromUnixTimestamp(1718744252) STEP 3600
)
GROUP BY project, activityType
ORDER BY total_count DESC
LIMIT 10


SELECT 
    groupArray((timestamp, count)) as timeseries,
            sum(count) as total_count

FROM (
    SELECT 
        toStartOfInterval(timestamp, interval 3600 second) as timestamp,
        count(*) as count
    FROM MooseActivityAugmented
    GROUP BY timestamp
    ORDER BY timestamp ASC
    WITH FILL FROM toStartOfInterval(fromUnixTimestamp(1718139452), interval 3600 second) TO fromUnixTimestamp(1718744252) STEP 3600
)
ORDER BY total_count DESC
LIMIT 10

SELECT name, type 
      FROM system.columns 
      WHERE table = 'sessions'

SELECT * FROM PageViewRaw