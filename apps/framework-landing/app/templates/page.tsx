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
      <Section>
        <Grid className="gap-y-5">
          <ThirdWidthContentContainer className="xl:order-1">
            <PlaceholderImage className=" bg-muted aspect-[4/3]" />
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="xl:order-4">
            <Heading> {content.templates[0].title} </Heading>
            <Text> {content.templates[0].description} </Text>
            <CTABar>
              <Link
                className="flex flex-col"
                href={content.templates[0].cta.href}
              >
                <CTAButton variant={"outline"}>
                  {content.templates[0].cta.label}
                </CTAButton>
              </Link>
            </CTABar>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className=" xl:m-0 xl:order-1">
            <PlaceholderImage className=" bg-muted aspect-[4/3]" />
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className=" xl:m-0  xl:order-4">
            <Heading> {content.templates[1].title} </Heading>
            <Text> {content.templates[1].description} </Text>
            <CTABar>
              <Link
                className="flex flex-col"
                href={content.templates[1].cta.href}
              >
                <CTAButton variant={"outline"}>
                  {content.templates[1].cta.label}
                </CTAButton>
              </Link>
            </CTABar>
          </ThirdWidthContentContainer>

          <ThirdWidthContentContainer className="xl:order-1">
            <PlaceholderImage className=" bg-muted aspect-[4/3]" />
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer className="xl:order-4">
            <Heading> {content.templates[2].title} </Heading>
            <Text> {content.templates[2].description} </Text>
            <CTABar>
              <Link
                className="flex flex-col"
                href={content.templates[2].cta.href}
              >
                <CTAButton variant={"outline"}>
                  {content.templates[2].cta.label}
                </CTAButton>
              </Link>
            </CTABar>
          </ThirdWidthContentContainer>
        </Grid>
      </Section>
      <FooterSection />
      <EmailSection />
    </>
  );
}
