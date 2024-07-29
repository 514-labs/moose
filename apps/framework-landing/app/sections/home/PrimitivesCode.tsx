"use client";

import {
  Tabs,
  TabsList,
  TabsContent,
} from "@514labs/design-system-components/components";

import { TrackableTabsTrigger } from "../../trackable-components";

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
} from "lucide-react";
import { useState, Fragment } from "react";
import CodeBlock from "../../shiki";

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
    description:
      "Codify the shape and structure of the data that is used in your application",
    filename: "/datamodels/models.ts",
    typescript: `
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
  },
  functions: {
    title: "Functions",
    description:
      "Add custom logic to filter, enrich, and transform data in-stream",
    filename: "/functions/UserActivity__ParsedActivity.ts",
    typescript: `
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
  },
  blocks: {
    title: "Blocks",
    description:
      "Create views to slice, aggregate, and join data across rows and tables",
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
    description:
      "Define parameterized endpoints to dynamically fetch and serve real-time insights to your apps",
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
  const [activeTab, setActiveTab] = useState("models");

  const tabs = ["models", "functions", "blocks", "apis"];

  return (
    <Fragment>
      <Section className="2xl:max-w-6xl mx-auto flex flex-col items-center px-5 my-16 sm:my-64">
        <Heading
          level={HeadingLevel.l1}
          className="justify-center align-center text-center mb-24 sm:text-5xl px-40"
        >
          Data modeling, processing, ingestion, orchestration, streaming,
          storage, and APIs—unified.{" "}
          <span className="bg-[linear-gradient(150.33deg,_#641bff_-210.85%,_#1983ff_28.23%,_#ff2cc4_106.53%)] bg-clip-text text-transparent">
            All in pure TypeScript or Python.
          </span>
        </Heading>
      </Section>
      <Section className="mx-auto xl:max-w-screen-xl sm:px-6 lg:px-8">
        <Grid className="flex flex-col">
          <FullWidthContentContainer className="flex flex-col lg:flex-row gap-5 p-4 sm:p-6 border rounded-3xl h-1/2">
            <HalfWidthContentContainer className="flex flex-col gap-5 justify-start lg:w-1/2 w-full">
              <Heading level={HeadingLevel.l3} className="mb-0">
                Moose Primitives
              </Heading>
              <Text className="text-muted-foreground sm:text-base">
                Define your unique application logic for how data is ingested,
                processed, aggregated, and consumed for your use case
              </Text>
              <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList className="mx-auto w-full justify-start">
                  {tabs.map((tab) => (
                    <TrackableTabsTrigger
                      key={tab}
                      value={tab}
                      className="py-0 px-1"
                      name="Moose Primitives Code"
                      subject={tab}
                    >
                      <Text className="py-0 px-2 ">{content[tab]?.title}</Text>
                    </TrackableTabsTrigger>
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
            <HalfWidthContentContainer className="lg:w-2/3 w-full">
              <CodeBlock
                code={content[activeTab]?.typescript || ""}
                language={"typescript"}
                filename={content[activeTab]?.filename || ""}
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
