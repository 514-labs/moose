import {
  Section,
  FullWidthContentContainer,
  Grid,
  HalfWidthContentContainer,
  ThirdWidthContentContainer,
} from "@/components/containers/page-containers";

import { EmailSection } from "@/app/sections/EmailSection";
import Link from "next/link";
import { Display, Heading, Text } from "@/components/typography/standard";
import { CTABar, CTAButton, PlaceholderImage } from "@/app/page";
import { FooterSection } from "@/app/sections/FooterSection";
import { TemplatesSection } from "../sections/home/TemplatesSection";

export default function TemplatesPage() {
  const content = {
    title: "Templates",
    hook: {
      title: "Get started in no time",
      description:
        "Jobs, pipelines, streams, data models, tables, views, schemas, APIs, and SDKs -- no more coordinating a tangled web of individual components. With a framework-based approach, each component is aware of the bigger picture, so your data stack is easier to manage and more resilient to change.",
    },
    templates: [
      {
        title: "Product Analytics",
        description:
          "Run stacks locally: see and test the impact of changes in real time as you edit code.",
        cta: {
          action: "cta-product-analytics-template-view",
          label: "Learn More",
          href: "/templates/product-analytics",
        },
      },
      {
        title: "LLM Application",
        description:
          "Run stacks locally: see and test the impact of changes in real time as you edit code.",
        cta: {
          action: "cta-product-analytics-template-view",
          label: "Learn More",
          href: "/templates/llm-application",
        },
      },
      {
        title: "Data Warehouse",
        description:
          "Run stacks locally: see and test the impact of changes in real time as you edit code.",
        cta: {
          action: "cta-product-analytics-template-view",
          label: "Learn More",
          href: "/templates/data-warehouse",
        },
      },
    ],
  };

  return (
    <>
      <Section>
        <FullWidthContentContainer>
          <Display>{content.title}</Display>
        </FullWidthContentContainer>
      </Section>
      <Section>
        <Grid>
          <HalfWidthContentContainer>
            <Heading>{content.hook.title}</Heading>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer>
            <Text>{content.hook.description}</Text>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
      <TemplatesSection />
      <FooterSection />
      <EmailSection />
    </>
  );
}
