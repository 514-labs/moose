"use client";

import {
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
} from "@514labs/design-system-components/components";
import {
  Heading,
  Text,
  HeadingLevel,
  SmallText,
  GradientText,
} from "@514labs/design-system-components/typography";
import {
  FullWidthContentContainer,
  Grid,
  Section,
  HalfWidthContentContainer,
} from "@514labs/design-system-components/components/containers";

import { Repeat, ArrowRight, Folders, Table, GitFork } from "lucide-react";
import { useState } from "react";

const content: {
  [key: string]: {
    title: string;
    description: string;
    filename: string;
    typescript: string;
  };
} = {
  models: {
    title: "Models",
    description: "Define a schema for raw data sources to ingest",
    filename: "/datamodels/models.ts",
    typescript: `
export interface UserActivity {
    id: string;
    userId: string;
    activity: string;
    timestamp: Date;
}`,
  },
  functions: {
    title: "Functions",
    description: "Add custom logic to augment & transform data in-stream",
    filename: "/functions/UserActivity__ParsedActivity.ts",
    typescript: `
import { UserActivity, ParsedActivity } from "/datamodels/models";

export default function convertUtc(source: UserActivity): ParsedActivity {
  return {
    id: source.id,
    userId: "puid" + source.userId,
    activity: source.activity,
    timestamp: new Date(source.timestamp),
  };
} `,
  },
  blocks: {
    title: "Blocks",
    description: "Create views to slice, aggregate, & join tables",
    filename: "/blocks/dailyActiveUsers.ts",
    typescript: `
import { createAggregation } from "@514labs/moose-lib";

export default createAggregation({
  name: "DailyActiveUsers",
  select: \` 
    SELECT 
        uniqState(userId) as dailyActiveUsers,
        toStartOfDay(timestamp) as date
    FROM ParsedActivity
    WHERE activity = 'Login' 
    GROUP BY toStartOfDay(timestamp)
    \`,
  orderBy: "date",
});`,
  },
  apis: {
    title: "APIs",
    description: "Fetch & serve real-time insights to your apps",
    filename: "/apis/dailyActiveUsers.ts",
    typescript: `
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
  },
};

const infrastructure = [
  {
    title: "Ingest API",
    infra: "Webserver",
    icon: <ArrowRight />,
    primitive: "models",
  },
  {
    title: "Topics",
    infra: "Streaming",
    icon: <Folders />,
    primitive: "models",
  },
  {
    title: "Processes",
    infra: "Orchestrator",
    icon: <Repeat />,
    primitive: "functions",
  },
  {
    title: "Tables",
    infra: "OLAP Storage",
    icon: <Table />,
    primitive: "models",
  },
  {
    title: "Views",
    infra: "OLAP Storage",
    icon: <GitFork />,
    primitive: "blocks",
  },
  {
    title: "Consumption API",
    infra: "Webserver",
    icon: <ArrowRight />,
    primitive: "apis",
  },
];

export const PrimitivesCode = () => {
  const [activeTab, setActiveTab] = useState("models");

  const tabs = ["models", "functions", "blocks", "apis"];

  return (
    <Section className="mx-auto xl:max-w-screen-xl px-0">
      <Heading
        level={HeadingLevel.l2}
        className="justify-center align-center text-center mb-24"
      >
        Moose provides the tools to manage your entire data lifecycle—from
        ingestion to consumption and everything in between.
        <br />
        <GradientText>All in pure TypeScript or Python.</GradientText>
      </Heading>
      <Grid className="flex flex-col">
        <FullWidthContentContainer className="flex flex-row gap-0 space-between-0 p-5 border rounded-3xl h-[450px]">
          <HalfWidthContentContainer className="flex flex-col gap-5 justify-start w-1/2 pr-5">
            <Heading level={HeadingLevel.l3} className="mb-0">
              Moose Primitives
            </Heading>
            <Text className="text-muted-foreground">
              Define how data is ingested, processed, aggregated, and consumed
              for your application
            </Text>
            <Tabs value={activeTab} onValueChange={setActiveTab}>
              <TabsList className="py-0 text-primary">
                {tabs.map((tab) => (
                  <TabsTrigger key={tab} value={tab} className="py-0">
                    <Text className="py-0 px-2">{content[tab]?.title}</Text>
                  </TabsTrigger>
                ))}
              </TabsList>
              {tabs.map((tab) => (
                <TabsContent key={tab} value={tab}>
                  <Text className="text-muted-foreground">
                    {content[tab]?.description}
                  </Text>
                </TabsContent>
              ))}
            </Tabs>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer className="w-1/2 mr-0">
            <code>
              <pre className="overflow-auto bg-primary/10 rounded-3xl h-full w-full px-5">
                {content[activeTab]?.typescript}
              </pre>
            </code>
          </HalfWidthContentContainer>
        </FullWidthContentContainer>
        <FullWidthContentContainer className="flex flex-col gap-2.5 border p-5 rounded-3xl">
          <Heading level={HeadingLevel.l3} className="mb-0">
            Moose Provisioned Infra
          </Heading>
          <Text className="text-muted-foreground">
            Moose translates the implemented primitives and configures your
            infrastructure for you as you develop
          </Text>
          <div className="w-full flex flex-row gap-x-2 space-x-0">
            {infrastructure.map((infra) => (
              <div
                className={
                  infra.primitive === activeTab
                    ? "bg-primary/10 px-5 py-2.5 rounded-3xl shadow-sm w-1/6"
                    : "px-5 py-2.5 w-1/6"
                }
                key={infra.title}
              >
                <div className="bg-primary/10 p-5 w-fit rounded-3xl">
                  {infra.icon}
                </div>
                <SmallText className="text-md">{infra.title}</SmallText>
                <SmallText className="text-muted-foreground text-wrap">
                  {infra.infra}
                </SmallText>
              </div>
            ))}
          </div>
        </FullWidthContentContainer>
      </Grid>
    </Section>
  );
};
