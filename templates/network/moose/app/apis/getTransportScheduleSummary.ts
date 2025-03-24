import {
  createConsumptionApi,
  ConsumptionHelpers as CH,
} from "@514labs/moose-lib";
import { tags } from "typia";

// Define expected parameters and their types
interface QueryParams {
  scheduleType?: string;
  dateFrom?: string;
  dateTo?: string;
}

// createConsumptionApi uses compile time code generation to generate a parser for QueryParams
export default createConsumptionApi<QueryParams>(
  async ({ scheduleType, dateFrom, dateTo }, { client, sql }) => {
    // Build WHERE clause for filtering
    let whereClause = sql`WHERE 1=1`;

    if (scheduleType) {
      whereClause = sql`${whereClause} AND scheduleType = ${scheduleType}`;
    }

    if (dateFrom) {
      whereClause = sql`${whereClause} AND departureTime >= ${dateFrom}`;
    }

    if (dateTo) {
      whereClause = sql`${whereClause} AND departureTime <= ${dateTo}`;
    }

    // Get total count of schedules
    const countResult = await client.query.execute(sql`
      SELECT count(*) as totalSchedules
      FROM TransportSchedule_0_0
      ${whereClause}
    `);

    const totalSchedules = countResult[0]?.totalSchedules || 0;

    // Get breakdown by schedule type
    const scheduleTypes = await client.query.execute(sql`
      SELECT 
        scheduleType,
        count(*) as scheduleCount,
        ROUND(count(*) * 100.0 / (
          SELECT count(*) 
          FROM TransportSchedule_0_0 
          ${whereClause}
        ), 1) as percentage
      FROM TransportSchedule_0_0
      ${whereClause}
      GROUP BY scheduleType
      ORDER BY scheduleCount DESC
    `);

    // Get average transit time in minutes
    const transitTimeResult = await client.query.execute(sql`
      SELECT 
        ROUND(avg(dateDiff('minute', departureTime, arrivalTime))) as avgTransitMinutes,
        min(dateDiff('minute', departureTime, arrivalTime)) as minTransitMinutes,
        max(dateDiff('minute', departureTime, arrivalTime)) as maxTransitMinutes
      FROM TransportSchedule_0_0
      ${whereClause}
    `);

    const transitTime = transitTimeResult[0] || {
      avgTransitMinutes: 0,
      minTransitMinutes: 0,
      maxTransitMinutes: 0,
    };

    // Get schedules by time of day
    const timeOfDayData = await client.query.execute(sql`
      SELECT 
        multiIf(
          toHour(departureTime) >= 5 AND toHour(departureTime) < 12, 'Morning (5am-12pm)',
          toHour(departureTime) >= 12 AND toHour(departureTime) < 17, 'Afternoon (12pm-5pm)',
          toHour(departureTime) >= 17 AND toHour(departureTime) < 21, 'Evening (5pm-9pm)',
          'Night (9pm-5am)'
        ) as timeOfDay,
        count(*) as scheduleCount
      FROM TransportSchedule_0_0
      ${whereClause}
      GROUP BY timeOfDay
      ORDER BY scheduleCount DESC
    `);

    // Get top 5 origin-destination pairs
    const topRoutes = await client.query.execute(sql`
      SELECT 
        origin,
        destination,
        count(*) as scheduleCount
      FROM TransportSchedule_0_0
      ${whereClause}
      GROUP BY origin, destination
      ORDER BY scheduleCount DESC
      LIMIT 5
    `);

    // Return consolidated response
    return {
      summary: {
        totalSchedules,
        appliedFilters: {
          scheduleType: scheduleType || null,
          dateRange:
            dateFrom || dateTo
              ? {
                  from: dateFrom || null,
                  to: dateTo || null,
                }
              : null,
        },
      },
      transitStats: transitTime,
      scheduleTypes,
      timeOfDay: timeOfDayData,
      topRoutes,
    };
  },
);
