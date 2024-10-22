"use client";

import {
  Tabs,
  TabsList,
  TabsContent,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  BackgroundIcon,
} from "@514labs/design-system-components/components";
import { TrackableTabsTrigger } from "@514labs/design-system-components/trackable-components";
import { CopyButton } from "../../copy-button";
import { cn } from "@514labs/design-system-components/utils";
import {
  Heading,
  Text,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import {
  FullWidthContentContainer,
  Grid,
  Section,
} from "@514labs/design-system-components/components/containers";

import {
  HardDriveDownload,
  RectangleEllipsis,
  Table,
  Code,
  Box,
  HardDriveUpload,
  CopyIcon,
} from "lucide-react";
import { useState } from "react";
import CodeBlock from "../../shiki";

const content = {
  models: {
    title: "Models",
    description:
      "Codify the shape and structure of the data that is used in your application",
    filename: "/datamodels/models",
    ts: `
import { Key } from "@514labs/moose-lib"

export interface UserActivity {
    id: Key<string>;
    userId: string;
    activity: string;
    timestamp: Date;
}

export interface ParsedActivity {
    id: Key<string>;
    userId: string;
    activity: string;
    utcTimestamp: Date;
}`,
    py: `
from moose_lib import Key, moose_data_model

@moose_data_model
@dataclass
class UserActivity:
    eventId: Key[str]
    timestamp: str
    userId: str
    activity: str

@moose_data_model
@dataclass
class ParsedActivity:
    eventId: Key[str]
    timestamp: datetime
    userId: str
    activity: str
`,
  },
  functions: {
    title: "Functions",
    description:
      "Implement custom processing functions to run on your data in-stream",
    filename: "/functions/UserActivity__ParsedActivity",
    ts: `
import { UserActivity } from "/datamodels/models"; 
import { ParsedActivity } from "/datamodels/models";

export default function run(source: UserActivity): ParsedActivity {
  return {
    id: source.id,
    userId: "puid" + source.userId,
    activity: source.activity,
    timestamp: new Date(source.timestamp),
  };
} `,
    py: `
from app.datamodels.models import UserActivity, ParsedActivity
from moose_lib import StreamingFunction

def parse_activity(activity: UserActivity) -> ParsedActivity:
    return ParsedActivity(
        eventId=activity.eventId,
        timestamp=datetime.fromisoformat(activity.timestamp),
        userId=activity.userId,
        activity=activity.activity,
    )

my_flow = StreamingFunction(
    run=parse_activity
)
`,
  },
  blocks: {
    title: "Blocks",
    description:
      "Create views to slice, aggregate, and join data across rows and tables",
    filename: "/blocks/dailyActiveUsers",
    ts: `
import {
  createAggregation,
  Blocks,
  ClickHouseEngines,
} from "@514labs/moose-lib";
 
const DESTINATION_TABLE = "UserActivitySummary";
const MATERIALIZED_VIEW = "UserActivitySummaryMV";
 
const selectQuery = \`
  SELECT 
    activity,
    uniqState(userId) as unique_user_count, 
    countState(activity) AS activity_count 
  FROM 
    ParsedActivity_0_0 
  GROUP BY 
    activity
\`;
 
export default {
  setup: createAggregation({
    tableCreateOptions: {
      name: DESTINATION_TABLE, 
      columns: {
        activity: "String",
        unique_user_count: "AggregateFunction(uniq, String)",
        activity_count: "AggregateFunction(count, String)", 
      },
      orderBy: "activity",
      engine: ClickHouseEngines.AggregatingMergeTree,
    },
    materializedViewName: MATERIALIZED_VIEW,
    select: selectQuery,
  }),
} as Blocks;`,
    py: `
from moose_lib import Blocks

destination_table = "DailyActiveUsers"
materialized_view = "DailyActiveUsers_mv"

select_sql = """
SELECT
  toStartOfDay(timestamp) as date,
  uniqState(userId) as dailyActiveUsers
FROM ParsedActivity_0_0
WHERE activity = 'Login'
GROUP BY toStartOfDay(timestamp)
"""

teardown_queries = [
    f"""
    DROP VIEW IF EXISTS {materialized_view}
    """,
    f"""
    DROP TABLE IF EXISTS {destination_table}
    """
]

setup_queries = [
    f"""
    CREATE TABLE IF NOT EXISTS {destination_table}
    (
        date Date,
        dailyActiveUsers AggregateFunction(uniq, String)
    )
    ENGINE = AggregatingMergeTree()
    ORDER BY date
    """,
    f"""
    CREATE MATERIALIZED VIEW IF NOT EXISTS {materialized_view}
    TO {destination_table}
    AS {select_sql}
    """,
    f"""
    INSERT INTO {destination_table}
    {select_sql}
    """
]

block = Blocks(teardown=teardown_queries, setup=setup_queries)
`,
  },
  apis: {
    title: "APIs",
    description:
      "Define parameterized endpoints to dynamically fetch and serve real-time insights to your apps",
    filename: "/apis/dailyActiveUsers",
    ts: `
interface QueryParams {
  limit: string;
  minDailyActiveUsers: string;
}
 
export default async function handle(
  { limit = "10", minDailyActiveUsers = "0" }: QueryParams,
  { client, sql },
) {
  return client.query(
    sql\`SELECT 
      date,
      dailyActiveUsers
    FROM DailyActiveUsers
    WHERE 
      dailyActiveUsers >= \${parseInt(minDailyActiveUsers)}
    LIMIT \${parseInt(limit)}\`,
  );
}
    `,
    py: `
def run(client, params):
    minDailyActiveUsers = int(params.get('minDailyActiveUsers', [0])[0])
    limit = int(params.get('limit', [10])[0])

    return client.query(
        '''SELECT
            date,
            uniqMerge(dailyActiveUsers) as dailyActiveUsers
        FROM DailyActiveUsers
        GROUP BY date
        HAVING dailyActiveUsers >= {minDailyActiveUsers}
        ORDER BY date
        LIMIT {limit}''',
        {
            "minDailyActiveUsers": minDailyActiveUsers,
            "limit": limit
        }
    )
`,
  },
};

const infrastructure = [
  {
    title: "Ingest API",
    infra: "Webserver",
    icon: HardDriveDownload,
    primitive: "models",
    order: "order-first",
  },
  {
    title: "Topics",
    infra: "Streams",
    icon: RectangleEllipsis,
    primitive: "models",
    order: "order-2",
  },
  {
    title: "Tables",
    infra: "OLAP DB",
    icon: Table,
    primitive: "models",
    order: "order-4",
  },
  {
    title: "Tasks",
    infra: "Orchestrator",
    icon: Code,
    primitive: "functions",
    order: "order-3",
  },
  {
    title: "Views",
    infra: "OLAP DB",
    icon: Box,
    primitive: "blocks",
    order: "order-5",
  },
  {
    title: "Egress API",
    infra: "Webserver",
    icon: HardDriveUpload,
    primitive: "apis",
    order: "order-6",
  },
];

export const PrimitivesCode = () => {
  const [language, setLanguage] = useState("ts");

  return (
    <Section className="mx-auto max-w-5xl sm:px-6 lg:px-8">
      <Heading
        level={HeadingLevel.l2}
        className="justify-center align-center text-center mb-24 sm:text-5xl"
      >
        Write code, get infra. Moose uses framework-defined infrastucture to
        keep you focused.
      </Heading>
      <FullWidthContentContainer className="w-full justify-center">
        <Tabs defaultValue="models">
          <TabsList className="mx-auto w-full justify-center">
            {Object.keys(content).map((tab) => (
              <TrackableTabsTrigger
                name={"primitives-code-snippet"}
                subject={tab}
                key={tab}
                value={tab}
                className="rounded-2xl"
              >
                <Text>{content[tab as keyof typeof content]?.title}</Text>
              </TrackableTabsTrigger>
            ))}
          </TabsList>
          {Object.keys(content).map((tab) => (
            <TabsContent key={tab} value={tab}>
              <Grid>
                <div className="col-span-* md:col-span-7 w-full flex flex-col gap-2">
                  <Heading level={HeadingLevel.l4}>
                    Develop application logic
                  </Heading>
                  <div className="relative h-[450px] w-full">
                    <div className="flex flex-row justify-end gap-2 items-center absolute top-4 right-4 z-10">
                      <Select value={language} onValueChange={setLanguage}>
                        <SelectTrigger className="px-2 py-1 w-fit justify-between gap-2 border text-primary bg-background">
                          <SelectValue placeholder="Select Language" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="ts">TypeScript</SelectItem>
                          <SelectItem value="py">Python</SelectItem>
                        </SelectContent>
                      </Select>
                      <CopyButton
                        copyText={
                          content[tab as keyof typeof content]?.[
                            language as "ts" | "py"
                          ] || ""
                        }
                        subject={content[tab as keyof typeof content]?.filename}
                        name={content[tab as keyof typeof content]?.filename}
                        className="px-2 py-1 w-fit justify-between gap-2 border-muted bg-transparent hover:bg-primary/10"
                      >
                        <CopyIcon size={16} />
                      </CopyButton>
                    </div>
                    <CodeBlock
                      className="pt-4"
                      code={
                        content[tab as keyof typeof content]?.[
                          language as "ts" | "py"
                        ] || ""
                      }
                      language={language as "ts" | "py"}
                      filename={
                        `${content[tab as keyof typeof content]?.filename}.${language}` ||
                        ""
                      }
                    />
                  </div>
                </div>

                <div className="sm:col-span-12 md:col-span-5 flex flex-col gap-2 w-full">
                  <Heading className="col-span-5" level={HeadingLevel.l4}>
                    Moose derives infra
                  </Heading>
                  <div className="col-span-5 flex flex-col gap-2">
                    {infrastructure.map((infra) => (
                      <div
                        key={infra.title}
                        className={cn(
                          "flex flex-row items-start justify-start gap-4 rounded-2xl p-1",
                          tab === infra.primitive ? "bg-muted" : "",
                          infra.order,
                        )}
                      >
                        <BackgroundIcon Icon={infra.icon} variant="default" />
                        <div>
                          <Text className="my-0">{infra.title}</Text>
                          <Text className="my-0 text-muted-foreground">
                            {infra.infra}
                          </Text>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </Grid>
            </TabsContent>
          ))}
        </Tabs>
      </FullWidthContentContainer>
    </Section>
  );
};
