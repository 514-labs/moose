import {
  Section,
  Grid,
  ThirdWidthContentContainer,
  HalfWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { IconCard } from "@514labs/design-system-components/components";
import {
  Bot,
  Cloud,
  Database,
  GitBranch,
  GitPullRequest,
  LineChart,
  Rocket,
  Wand2,
} from "lucide-react";

export function MooseIngressProp() {
  return (
    <Section className="max-w-5xl mx-auto">
      <Heading
        level={HeadingLevel.l2}
        className="justify-center align-center text-center mb-24 sm:text-5xl"
      >
        Get data into your application.
        <span className="text-muted-foreground">
          {" "}
          Scalable data ingress that ensures the right data is captured
        </span>
      </Heading>
      <Grid>
        <HalfWidthContentContainer>
          <IconCard
            title="Connect to any source realtime or on a schedule"
            description="Easily create connectors that work reliably and scale easily"
            Icon={GitBranch}
          />
        </HalfWidthContentContainer>
        <HalfWidthContentContainer>
          <IconCard
            title="Data validation at ingest that scales"
            description="Validate schemas and easily remediate data quality issues"
            Icon={LineChart}
          />
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
}

export function MooseEgressProp() {
  return (
    <Section className="max-w-5xl mx-auto">
      <Heading
        level={HeadingLevel.l2}
        className="justify-center align-center text-center mb-24 sm:text-5xl"
      >
        Turn data into a product for your users.
        <span className="text-muted-foreground">
          {" "}
          Create views and APIs to quickly turn data into insights
        </span>
      </Heading>
      <Grid>
        <HalfWidthContentContainer>
          <IconCard
            title="Turn data into an API for apps and services"
            description="Easily create scalable egress APIs to make your data accessible"
            Icon={GitPullRequest}
          />
        </HalfWidthContentContainer>
        <HalfWidthContentContainer>
          <IconCard
            title="Create views & process data in your database"
            description="Leverage your OLAP database to slice and dice your data at scale"
            Icon={Database}
          />
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
}

export function MooseValuePropSection() {
  return (
    <Section className="max-w-5xl mx-auto">
      <Heading
        level={HeadingLevel.l2}
        className="justify-center align-center text-center mb-24 sm:text-5xl"
      >
        Tools you love, context you need.
        <span className="bg-gradient bg-clip-text text-transparent">
          {" "}
          Built for modern, AI-centric development workflows
        </span>
      </Heading>
      <Grid>
        <ThirdWidthContentContainer>
          <IconCard
            title="Local to cloud development workflows"
            description="Build your app locally using the tools you love then take your app to the cloud reliably"
            Icon={Cloud}
          />
        </ThirdWidthContentContainer>
        <ThirdWidthContentContainer>
          <IconCard
            title="Data Model awareness for you and your AI co-pilots"
            description="Data context pulled directly into your development workflow at dev and build time"
            Icon={Wand2}
          />
        </ThirdWidthContentContainer>
        <ThirdWidthContentContainer>
          <IconCard
            title="Horizontal and vertical scalability"
            description="Moose is multithreaded and scales to multiple machines to maximize scale"
            Icon={Rocket}
          />
        </ThirdWidthContentContainer>
      </Grid>
    </Section>
  );
}
