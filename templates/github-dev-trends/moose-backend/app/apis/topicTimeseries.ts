import {
  ConsumptionApi,
  ConsumptionUtil,
  ConsumptionHelpers as CH,
} from "@514labs/moose-lib";
import { tags } from "typia";
import { watchEventPipeline } from "../ingest/WatchEvent";

interface QueryParams {
  interval?: "minute" | "hour" | "day";
  limit?: number & tags.Minimum<1> & tags.Type<"int32">;
  exclude?: string & tags.Pattern<"^([^,]+)(,[^,]+)*$">; // comma separated list of tags to exclude
}

interface ResponseBody {
  time: string;
  topicStats: {
    topic: string;
    eventCount: number;
    uniqueRepos: number;
    uniqueUsers: number;
  }[];
}

export default new ConsumptionApi<QueryParams, ResponseBody[]>(
  "topicTimeseries",
  async (
    { interval = "minute", limit = 10, exclude = "" }: QueryParams,
    { client, sql }: ConsumptionUtil,
  ) => {
    const intervalMap = {
      hour: {
        select: sql`toStartOfHour(createdAt) AS time`,
        groupBy: sql`GROUP BY time, topic`,
        orderBy: sql`ORDER BY time, totalEvents DESC`,
        limit: sql`LIMIT ${limit} BY time`,
      },
      day: {
        select: sql`toStartOfDay(createdAt) AS time`,
        groupBy: sql`GROUP BY time, topic`,
        orderBy: sql`ORDER BY time, totalEvents DESC`,
        limit: sql`LIMIT ${limit} BY time`,
      },
      minute: {
        select: sql`toStartOfFifteenMinutes(createdAt) AS time`,
        groupBy: sql`GROUP BY time, topic`,
        orderBy: sql`ORDER BY time, totalEvents DESC`,
        limit: sql`LIMIT ${limit} BY time`,
      },
    };

    const query = sql`
            SELECT
                time,
                arrayMap(
                    (topic, events, repos, users) -> map(
                        'topic', topic,
                        'eventCount', toString(events),
                        'uniqueRepos', toString(repos),
                        'uniqueUsers', toString(users)
                    ),
                    groupArray(topic),
                    groupArray(totalEvents),
                    groupArray(uniqueReposCount),
                    groupArray(uniqueUsersCount)
                ) AS topicStats
            FROM (
                SELECT
                    ${intervalMap[interval].select},
                    arrayJoin(repoTopics) AS topic,
                    count() AS totalEvents,
                    uniqExact(repoId) AS uniqueReposCount,
                    uniqExact(actorId) AS uniqueUsersCount
                FROM ${CH.table(watchEventPipeline.table!.name)}
                WHERE length(repoTopics) > 0
                ${exclude ? sql`AND arrayAll(x -> x NOT IN (${exclude}), repoTopics)` : sql``}
                ${intervalMap[interval].groupBy}
                ${intervalMap[interval].orderBy}
                ${intervalMap[interval].limit}
            )
            GROUP BY time
            ORDER BY time;
        `;

    const resultSet = await client.query.execute(query);
    const data = (await resultSet.json()) as ResponseBody[];
    return data;
  },
);
