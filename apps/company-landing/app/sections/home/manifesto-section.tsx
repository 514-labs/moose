import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";
import { Text } from "design-system/typography";
import { PlaceholderImage } from "../../../components.tsx/PlaceholderImage";

export const ManifestoSection = () => {
  return (
    <Section>
      <Grid>
        <HalfWidthContentContainer className="2xl:col-span-3">
          <PlaceholderImage className="aspect-square bg-muted sticky top-20" />
        </HalfWidthContentContainer>
        <HalfWidthContentContainer className="2xl:col-start-7">
          <Text className="mt-0">
            Let&apos;s be honest. Developing any kind of data-driven or
            analytics-driven application on a modern data stack is, let&apos;s
            say, <span className=" line-through">a complete disaster</span> “far
            from an ideal experience.”
          </Text>
          <Text>
            Let&apos;s be honest. Software development in the world of data
            engineering is far from an ideal experience.
          </Text>
          <Text>
            To do anything interesting with data, you have to immerse yourself
            in a whole specialized ecosystem of solutions, from storage (e.g.
            snowflake, s3), to streaming (e.g. kafka), to processing (eg. spark,
            python scripts), to ingest (eg. fivetran, airbyte), to modeling (eg.
            dbt), to consumption (eg. views, data product APIs), to
            orchestration (eg. airflow). You have to hire as many data
            scientists, data analysts, and/or data engineers as you can
            manage—or you and your engineering team have to learn a whole new
            paradigm yourselves. If you can afford the slew of commercial
            solutions, great; otherwise, you&apos;re piecing together your own
            open source frankenstein. And then, just to get started, you have to
            create and configure tables, views, topics, jobs, scripts,
            connectors, APIs, and SDKs - and string together all these different
            components in production to all play nicely with each other. And god
            help you when someone changes something upstream, or you have to
            coordinate handoffs across specialized teams.
          </Text>

          <Text>
            This isn&apos;t how modern software development is supposed to be!
            By comparison, the alternate worlds of web apps, mobile apps, and
            backend services make software development look like an absolute joy
            of rainbows and roses. Software developers don&apos;t manage
            fragmented components in isolation, crossing their fingers and
            hoping they all play nice. They create holistic applications, with
            contracts between microservices. They leverage frameworks that
            abstract away infrastructure and middleware complexity. Software
            developers don&apos;t manually manage components in production and
            in the cloud, spinning plates to keep everything alive. They write
            code. And they leverage decades of best practices around code for
            speed, quality and collaboration - best practices like version
            control, local development, CI/CD, automated testing, change
            management, and devops.
          </Text>

          <Text>
            Why should development on an analytics/data stack be any different?
            It shouldn&apos;t!
          </Text>

          <Text>
            Our mission is to bring incredible developer experiences to the data
            stack. We believe that we&apos;ll have accomplished this when data
            or analytics can be dropped from the title of the people using data
            platforms. When data engineers are just engineers. When every
            developer can deliver high-quality data products, derive insights
            for themselves and their stakeholders, and easily integrate
            predictive and generative AI. When data quality is proactive and
            shifted left, not reactive and in production. When data services are
            just another software service - written in code, powered by
            microservices and frameworks, and managed with software development
            best practices. When becoming data-driven is second nature and
            something you can do without hiring an army of data and analytics
            engineers. When doing interesting things with data is
            cost-effective, and ROI is nearly guaranteed.
          </Text>

          <Text>
            Software has long been eating the world. It&apos;s high time for
            software to eat data engineering.
          </Text>

          <Text>
            Introducing MooseJS, an open source developer framework for your
            data & analytics stack. Moose unifies your data stack, unlocks
            software development best practices, and hopefully delights you, the
            developer. Moose is still in alpha with a lot more to come, so
            please check it out and let us know what you think. If you&apos;re
            interested in enterprise solutions, commercial support, or design
            partnerships, then we&apos;d love to chat with you.
          </Text>
        </HalfWidthContentContainer>
      </Grid>
    </Section>
  );
};
