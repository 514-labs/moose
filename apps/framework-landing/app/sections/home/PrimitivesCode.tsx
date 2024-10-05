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
  TrackableTabsTrigger,
} from "@514labs/design-system-components/components";

import { CopyButton } from "../../copy-button";

import {
  Heading,
  Text,
  HeadingLevel,
  SmallText,
} from "@514labs/design-system-components/typography";
import {
  FullWidthContentContainer,
  Grid,
  Section,
  HalfWidthContentContainer,
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
import { useState, Fragment } from "react";
import CodeBlock from "../../shiki";

const content = {
  models: {
    title: "Models",
    description:
      "Codify the shape and structure of the data that is used in your application",
    filename: "/datamodels/models",
    ts: `
export interface UserActivity {
    id: string;
    userId: string;
    activity: string;
    timestamp: Date;
}

export interface ParsedActivity {
    id: string;
    userId: string;
    activity: string;
    utcTimestamp: Date;
}`,
    py: `
from moose_lib import Key

@dataclass
class UserActivity:
    eventId: Key[str]
    timestamp: str
    userId: str
    activity: str

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
      "Add custom logic to filter, enrich, and transform data in-stream",
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
from moose_lib import Flow

def parse_activity(activity: UserActivity) -> ParsedActivity:
    return ParsedActivity(
        eventId=activity.eventId,
        timestamp=datetime.fromisoformat(activity.timestamp),
        userId=activity.userId,
        activity=activity.activity,
    )

my_flow = Flow(
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
    title: "Ingress Routes",
    infra: "Webserver",
    icon: <HardDriveDownload />,
    primitive: "models",
    order: "md:order-first",
  },
  {
    title: "Topics",
    infra: "Streams",
    icon: <RectangleEllipsis />,
    primitive: "models",
    order: "md:order-2",
  },
  {
    title: "Tables",
    infra: "OLAP DB",
    icon: <Table />,
    primitive: "models",
    order: "md:order-4",
  },
  {
    title: "Tasks",
    infra: "Orchestrator",
    icon: <Code />,
    primitive: "functions",
    order: "md:order-3",
  },
  {
    title: "Views",
    infra: "OLAP DB",
    icon: <Box />,
    primitive: "blocks",
    order: "md:order-5",
  },
  {
    title: "Egress Routes",
    infra: "Webserver",
    icon: <HardDriveUpload />,
    primitive: "apis",
    order: "md:order-6",
  },
];

export const PrimitivesCode = () => {
  const [activeTab, setActiveTab] = useState<keyof typeof content>("models");
  const [language, setLanguage] = useState("ts");

  return (
    <Fragment>
      <Section className="2xl:max-w-6xl mx-auto flex flex-col items-center px-5 my-16 sm:my-64">
        <Grid>
          <FullWidthContentContainer>
            <Heading
              level={HeadingLevel.l1}
              className="max-w-5xl justify-center align-center text-center md:mb-24 sm:text-5xl"
            >
              Data modeling, processing, ingestion, orchestration, streaming,
              storage, and APIs—unified.{" "}
              <span className="bg-[linear-gradient(150.33deg,_#641bff_-210.85%,_#1983ff_28.23%,_#ff2cc4_106.53%)] bg-clip-text text-transparent">
                All in pure TypeScript or Python.
              </span>
            </Heading>
          </FullWidthContentContainer>
        </Grid>
      </Section>
      <Section className="mx-auto max-w-5xl sm:px-6 lg:px-8">
        <Grid className="flex flex-col">
          <FullWidthContentContainer className="flex md:flex-row flex-col gap-5 p-4 sm:p-6 border rounded-3xl h-fit">
            <HalfWidthContentContainer className="flex flex-col gap-5 justify-start md:w-1/2 w-full">
              <Heading level={HeadingLevel.l3} className="mb-0">
                Moose Primitives
              </Heading>
              <Text className="text-muted-foreground">
                Define your unique application logic for how data is ingested,
                processed, aggregated, and consumed for your use case
              </Text>
              <Tabs
                value={activeTab}
                onValueChange={(tab) =>
                  setActiveTab(tab as keyof typeof content)
                }
              >
                <TabsList className="mx-auto w-full justify-start">
                  {Object.keys(content).map((tab) => (
                    <TrackableTabsTrigger
                      key={tab}
                      value={tab}
                      className="py-0 px-1"
                      name="Moose Primitives Code"
                      subject={tab}
                    >
                      <Text className="py-0 px-2 ">
                        {content[tab as keyof typeof content]?.title}
                      </Text>
                    </TrackableTabsTrigger>
                  ))}
                </TabsList>
                {Object.keys(content).map((tab) => (
                  <TabsContent key={tab} value={tab}>
                    <Text className="text-muted-foreground">
                      {content[tab as keyof typeof content]?.description}
                    </Text>
                  </TabsContent>
                ))}
              </Tabs>
            </HalfWidthContentContainer>
            <HalfWidthContentContainer className="md:w-2/3 w-full overflow-hidden">
              <div className="flex justify-end gap-2 items-center">
                <Select value={language} onValueChange={setLanguage}>
                  <SelectTrigger className="px-2 py-1 w-fit justify-between gap-2 border text-primary">
                    <SelectValue placeholder="Select Language" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="ts">TypeScript</SelectItem>
                    <SelectItem value="py">Python</SelectItem>
                  </SelectContent>
                </Select>
                <CopyButton
                  copyText={content[activeTab]?.[language as "ts" | "py"] || ""}
                  subject={content[activeTab]?.filename}
                  name={content[activeTab]?.filename}
                  className="px-2 py-1 w-fit justify-between gap-2 border-muted bg-transparent hover:bg-primary/10"
                >
                  <CopyIcon size={16} />
                </CopyButton>
              </div>
              <CodeBlock
                className="mt-2"
                code={content[activeTab]?.[language as "ts" | "py"] || ""}
                language={language as "ts" | "py"}
                filename={`${content[activeTab]?.filename}.${language}` || ""}
              />
            </HalfWidthContentContainer>
          </FullWidthContentContainer>
          <FullWidthContentContainer className="flex flex-col gap-2.5 border p-5 rounded-3xl justify-start text-left">
            <Heading level={HeadingLevel.l3} className="mb-0">
              Moose Provisioned Infra
            </Heading>
            <Text className="text-muted-foreground">
              Moose interprets the application logic in your primitives to
              automatically manage and configure assets in your underlying
              infrastructure
            </Text>
            <div className="w-full grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-4">
              {infrastructure.map((infra) => (
                <div
                  className={`px-4 pt-4 rounded-2xl ${
                    infra.primitive === activeTab
                      ? "bg-primary/10 shadow-sm"
                      : ""
                  } ${infra.order}`}
                  key={infra.title}
                >
                  <div className="bg-primary/10 p-3 w-fit rounded-xl">
                    {infra.icon}
                  </div>
                  <SmallText className="text-sm font-medium">
                    {infra.title}
                  </SmallText>
                  <SmallText className="text-muted-foreground text-wrap">
                    {infra.infra}
                  </SmallText>
                </div>
              ))}
            </div>
          </FullWidthContentContainer>
        </Grid>
      </Section>
    </Fragment>
  );
};
