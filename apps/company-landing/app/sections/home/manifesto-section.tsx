import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@514labs/design-system-components/components";
import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "@514labs/design-system-components/components/containers";
import { SmallText, Text } from "@514labs/design-system-components/typography";
import Image from "next/image";

export const ManifestoSection = () => {
  return (
    <Section>
      <Grid className="gap-y-5">
        <HalfWidthContentContainer className="lg:col-span-3 aspect-square bg-muted sticky md:top-24">
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
        <HalfWidthContentContainer className="lg:col-start-7">
          <Text className="mt-0">
            The modern data stack has come a long way.
          </Text>

          <Text>And yet...</Text>

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
            spinning plates to keep everything alive. They define their
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
            experiences to the data & analytics stack. We believe that
            we&apos;ll have accomplished this when data or analytics can be
            dropped from the title of the people using data platforms. When data
            engineers are just engineers. When data services are just another
            software service - written by software engineers, powered by
            microservices and frameworks, and managed with software development
            best practices. When every developer can deliver high-quality data
            products, derive insights for themselves and their stakeholders, and
            easily integrate predictive and generative AI. When becoming
            data-driven is second nature, and ROI on data investments is nearly
            guaranteed.
          </Text>

          <Text>
            Building software is a delight{" "}
            <HoverCard openDelay={300} closeDelay={300}>
              <HoverCardTrigger>
                <sup className=" cursor-pointer">*</sup>
              </HoverCardTrigger>
              <HoverCardContent className="p-5 w-96">
                <SmallText>
                  Except for{" "}
                  <a
                    href="https://twitter.com/secretGeek/status/7269997868"
                    className="underline"
                  >
                    two things
                  </a>{" "}
                  of course: naming things, cache invalidation and off by one
                  errors.
                </SmallText>
              </HoverCardContent>
            </HoverCard>
            . It&apos;s time to bring some delight to the data & analytics
            stack.
          </Text>

          <Text>
            —Tim, Nico, Alex, Chris, Dan, Carlos, Olivia, Jonathan, the Georges
            and Joj
          </Text>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
