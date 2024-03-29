import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";
import { Text } from "design-system/typography";
import Image from "next/image";

export const ManifestoSection = () => {
  return (
    <Section>
      <Grid className="gap-y-5">
        <HalfWidthContentContainer className="2xl:col-span-3 aspect-square bg-muted sticky md:top-24">
          <div className="relative h-full">
            <Image
              className=""
              priority
              src="/images/manifesto/mjs_img_5.webp"
              fill
              alt="girl"
              sizes=" (max-width: 768px) 150vw, 25vw"
            />
          </div>
        </HalfWidthContentContainer>
        <HalfWidthContentContainer className="2xl:col-start-7">
          <Text className="mt-0">
            Building a data-intensive application is still like needing to build
            a car to drive to a destination.
          </Text>

          <Text>
            To do anything interesting with data, you have to immerse yourself
            in a specialized ecosystem of solutions: from storage (e.g.
            snowflake, s3), to streaming (e.g. kafka), to processing (eg. spark,
            python scripts), to ingest (eg. fivetran, airbyte), to modeling (eg.
            dbt), to orchestration (eg. airflow). You have to hire as many data
            engineers as you can manage. You have to create data models, tables,
            views, topics, jobs, scripts, connectors, APIs, and SDKs - and then
            string them all together to play nicely in production. And good luck
            when something changes upstream, or you have to hand off work across
            teams.
          </Text>

          <Text>
            By comparison, developing web applications and services is a breeze.
          </Text>
          <Text>
            Software developers don&apos;t manage fragmented components in
            isolation and cross their fingers, hoping everything plays nicely.
            They create holistic applications, with contracts between services.
            They don&apos;t manually configure in production and in the cloud,
            spinning plates to keep everything alive. They build using the tools
            they love, locally and in the cloud. They define their
            infrastructure as code, and deploy with a single command. They use
            frameworks that abstract away infrastructure and middleware
            complexity. They leverage decades of software best practices for
            speed, quality and collaboration - like version control, CI/CD,
            automated testing, change management, and DevOps.
          </Text>

          <Text>
            Why should development on a data/analytics stack be any different?
            It shouldn&apos;t be!
          </Text>

          <Text>It. Should. Not. Be.</Text>

          <Text>
            Our mission at Fiveonefour is to bring incredible developer
            experiences to the data stack. We believe that we&apos;ll have
            accomplished this when data or analytics can be dropped from the
            title of the people using data platforms. When data engineers are
            just engineers. When data services are just another software service
            - written by software engineers, powered by microservices and
            frameworks, and managed with software development best practices.
            When every developer can deliver high-quality data products, derive
            insights for themselves and their stakeholders, and easily integrate
            predictive and generative AI. When becoming data-driven is second
            nature, and ROI on data investments is nearly guaranteed.
          </Text>

          <Text>
            Building software is delightful. It's time to bring some delight to
            the data & analytics stack.
          </Text>

          <Text>
            —Tim, Nico, Alex, Chris, Dan, Carlos, Olivia and the Georges
          </Text>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
