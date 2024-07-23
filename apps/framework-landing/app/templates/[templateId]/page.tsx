import Link from "next/link";

import { CTABar } from "../../page";
import FooterSection from "../../sections/FooterSection";
// import {
//   Accordion,
//   AccordionContent,
//   AccordionItem,
//   AccordionTrigger
// } from "@514labs/design-system-components/components";
import {
  Grid,
  Section,
  FullWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Display,
  Text,
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { TrackCtaButton } from "../../trackable-components";
import { Suspense } from "react";
import { TemplateImg } from "../../sections/home/TemplateImg";
import React from "react";
import { LooseMooseSection } from "../../sections/home/LooseMooseSection";
import { TrackingVerb } from "@514labs/event-capture/withTrack";

interface TemplateAccordionItem {
  title: string;
  steps: {
    title: string;
    description: string;
    command?: string;
    action?: {
      label: string;
      href: string;
    };
  }[];
}

interface TemplateAccordionProps {
  templateAccordionItems: TemplateAccordionItem[];
}

// function TemplateAccordion({ templateAccordionItems }: TemplateAccordionProps) {
//   return (
//     <Accordion
//       type="single"
//       defaultValue="item-0"
//       collapsible
//       className="w-full"
//     >
//       {templateAccordionItems.map((item, index) => (
//         <AccordionItem value={`item-${index}`} key={index}>
//           <AccordionTrigger>
//             <Text>{item.title}</Text>
//           </AccordionTrigger>
//           <AccordionContent>
//             {item.steps.map((step, index) => (
//               <div key={index} className="py-5">
//                 <Text className="text-muted-foreground">{step.title}</Text>
//                 <Text>{step.description}</Text>
//                 {step.command && (
//                   <TrackableCodeSnippet
//                     name={item.title}
//                     subject={step.command}
//                   >
//                     {step.command}
//                   </TrackableCodeSnippet>
//                 )}
//                 {step.action && (
//                   <Link href={step.action.href}>
//                     <TrackCtaButton
//                       name={item.title}
//                       subject={step.action.label}
//                     >
//                       {step.action.label}
//                     </TrackCtaButton>
//                   </Link>
//                 )}
//               </div>
//             ))}
//           </AccordionContent>
//         </AccordionItem>
//       ))}
//     </Accordion>
//   );
// }

// The layout for specific tempaltes
export default function TemplatePage({
  params,
}: {
  params: { templateId: string };
}) {
  const content = {
    templateDetails: [
      {
        slug: "product-analytics",
        title: "Product Analytics",
        imageSrcLight: "/images/diagrams/img-diagram-PA-light.svg",
        imageSrcDark: "/images/diagrams/img-diagram-PA-dark.svg",
        ctas: {
          docs: {
            label: "Get Started",
            href: "https://docs.moosejs.com/templates/product-analytics",
            type: "primary",
            action: TrackingVerb.clicked,
            name: "Template Get Started",
            subject: `Product Analytics`,
          },
          github: {
            label: "View Repository",
            href: "https://github.com/514-labs/moose/tree/main/templates/product-analytics",
            type: "secondary",
            action: TrackingVerb.clicked,
            name: "Template View Repository",
            subject: `Product Analytics`,
          },
          install: {
            text: "npx create-moose-app moose-product-analytics --template product-analytics",
            action: TrackingVerb.clicked,
            name: "Template Install Command",
            subject: `Product Analytics`,
          },
        },
        description:
          "Capture clickstream events from your users and analyze their interactions with your product",
        features: {
          title: "Features",
          items: [
            {
              title: "Analytics services",
              label: "Backend",
              items: ["MooseJS", "Event Data Models"],
            },
            {
              title: "Analytics dashboard",
              label: "Frontend",
              items: ["NextJS", "TailwindCSS", "Observable Plot"],
            },
            {
              title: "Analytics utilities",
              label: "Intrumentation",
              items: [
                "Event Capture SDK",
                "Page tacking",
                "Session management",
              ],
            },
          ],
        },
      },
      {
        slug: "llm-application",
        title: "LLM Application",
        imageSrcLight: "/images/diagrams/img-diagram-LLM-light.svg",
        imageSrcDark: "/images/diagrams/img-diagram-LLM-dark.svg",
        description:
          "Build natural language interfaces on top of your data by exposing data APIs to a LLM",
        usage: [],
      },
      {
        slug: "data-warehouse",
        title: "Data Warehouse",
        imageSrcLight: "/images/diagrams/img-diagram-DW-light.svg",
        imageSrcDark: "/images/diagrams/img-diagram-DW-dark.svg",
        description:
          "Integrate enterprise data sources into a central warehouse and connect to your BI tool of choice",
        usage: [],
      },
    ],
  };

  const template = content.templateDetails.find(
    (template) => template.slug === params.templateId,
  );

  return (
    <Grid>
      <div className="col-span-12 md:col-span-6 w-full relative mx-auto xl:max-w-screen-xl 2xl:px-10">
        <div className="md:sticky top-10 w-full relative mx-auto">
          <Section className="sm:pr=l-0 xl:pl-32 2xl:pl-48 3xl:pl-96 my-0">
            <div>
              <Link href="/templates">
                <Text className="mt-0 pt-0">
                  <span className="text-muted-foreground">Templates / </span>{" "}
                  <span> {template?.title} </span>
                </Text>
              </Link>
            </div>
            <Display>{template?.title}</Display>
            <Heading
              level={HeadingLevel.l2}
              className="text-muted-foreground pb-10"
            >
              {template?.description}
            </Heading>
            {template?.ctas ? (
              <div className="flex flex-col gap-5">
                <CTABar>
                  <Link href={template.ctas.docs.href}>
                    <TrackCtaButton
                      name={template.ctas.docs.name}
                      subject={template.ctas.docs.subject}
                      targetUrl={template.ctas.docs.href}
                    >
                      {template.ctas.docs.label}
                    </TrackCtaButton>
                  </Link>
                  <Link href={template.ctas.github.href}>
                    <TrackCtaButton
                      name={template.ctas.github.name}
                      subject={template.ctas.github.subject}
                      targetUrl={template.ctas.github.href}
                      variant="outline"
                    >
                      {template.ctas.github.label}
                    </TrackCtaButton>
                  </Link>
                </CTABar>
                {/* <TrackableCodeSnippet name={template.ctas.install.name} subject={template.ctas.install.subject}>{template.ctas.install.text}</TrackableCodeSnippet> */}
              </div>
            ) : (
              <Text className="text-muted-foreground">Coming Soon</Text>
            )}
            {/* <div className="py-10 grid gap-x-0 gap-y-0">
              {template?.features?.items.map((feature, index) => (
                <Grid key={index}>
                  <div key={index} className="col-span-6">
                    <Text className="my-0">{feature.title}</Text>
                    <Text className="my-0 text-muted-foreground">
                      {feature.label}
                    </Text>
                  </div>
                  <div className="col-span-6">
                    {feature.items.map((item, index) => (
                      <>
                        <Text className="my-0" key={index}>
                          {item}
                        </Text>
                      </>
                    ))}
                  </div>
                  {index < template.features.items.length - 1 && (
                    <div className="col-span-12">
                      <Separator className="my-3" />
                    </div>
                  )}
                </Grid>
              ))}
            </div> */}
          </Section>
        </div>
      </div>
      <div className="col-span-12 md:col-span-6 w-full relative mx-auto xl:max-w-screen-xl">
        <div className="mb-5">
          <Section className="sm:pr=l-0 xl:pr-32 2xl:pr-48 3xl:pr-96">
            <div className="aspect-[4/3] flex flex-col justify-center">
              <div className="relative h-4/5">
                {template && (
                  <Suspense fallback={<div>Loading...</div>}>
                    <TemplateImg
                      srcDark={template.imageSrcDark}
                      srcLight={template.imageSrcLight}
                      alt={template.title}
                    />
                  </Suspense>
                )}
              </div>
            </div>
          </Section>
        </div>
      </div>
      <FullWidthContentContainer className="col-span-12 ">
        <FooterSection />
        {/* <EmailSection /> */}
        <LooseMooseSection />
      </FullWidthContentContainer>
    </Grid>
  );
}
