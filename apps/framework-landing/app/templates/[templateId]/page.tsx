import Link from "next/link";

import { CTABar } from "../../page";
import FooterSection from "../../sections/FooterSection";
import { EmailSection } from "../../sections/EmailSection";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
  Separator,
} from "design-system/components";
import {
  Grid,
  Section,
  FullWidthContentContainer,
} from "design-system/components/containers";
import { Display, Text } from "design-system/typography";
import {
  TrackCtaButton,
  TrackableCodeSnippet,
} from "../../trackable-components";
import { CopyButton } from "./copy-button";

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

function TemplateAccordion({ templateAccordionItems }: TemplateAccordionProps) {
  return (
    <Accordion
      type="single"
      defaultValue="item-0"
      collapsible
      className="w-full"
    >
      {templateAccordionItems.map((item, index) => (
        <AccordionItem value={`item-${index}`} key={index}>
          <AccordionTrigger>
            <Text>{item.title}</Text>
          </AccordionTrigger>
          <AccordionContent>
            {item.steps.map((step, index) => (
              <div key={index} className="py-5">
                <Text className="text-muted-foreground">{step.title}</Text>
                <Text>{step.description}</Text>
                {step.command && (
                  <TrackableCodeSnippet
                    name={item.title}
                    subject={step.command}
                  >
                    {step.command}
                  </TrackableCodeSnippet>
                )}
                {step.action && (
                  <Link href={step.action.href}>
                    <TrackCtaButton
                      name={item.title}
                      subject={step.action.label}
                    >
                      {step.action.label}
                    </TrackCtaButton>
                  </Link>
                )}
              </div>
            ))}
          </AccordionContent>
        </AccordionItem>
      ))}
    </Accordion>
  );
}

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
        img: "/images/templates/mjs-img-product-1.svg",
        cta: {
          action: "cta-product-analytics-install",
          label: "Create Template Command",
          text: "npx create-moose-app your-analytics-app --template product-analytics",
        },
        description:
          "Harness the power of a full-stack, real-time user analytics platform designed for product analytics, powered by MooseJS and NextJS.",
        features: {
          title: "Features",
          items: [
            {
              title: "Analytics services",
              label: "Backend",
              items: ["MooseJS", "Event Data Models", "Bot Filtering Flow"],
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
                "Session managemment",
              ],
            },
          ],
        },
        usage: [
          {
            title: "Setting up locally",
            steps: [
              {
                title: "Step 1",
                description:
                  "Begin with installing the template on your machine",
                command:
                  "npx create-moose-app your-analytics-app --template product-analytics",
              },
              {
                title: "Step 2",
                description:
                  "Start Moose development environment from the moose directory",
                command: "cd moose && npx @514labs/moose-cli@latest dev",
              },
              {
                title: "Step 3",
                description:
                  "Install the dependencies and start the development server from the next directory",
                command: "cd next && npm install && npm run dev",
              },
              {
                title: "Step 4",
                description:
                  "Navigate to localhost:3001 to view the provided data models, flows, and insights.",
              },
              {
                title: "Step 5",
                description:
                  "Navigate to localhost:3000 to view your NextJS application.",
              },
            ],
          },
          {
            title: "Capturing Events",
            description:
              "You'll now need to capture events in your user facing applications. You can find endpoints and SDKs in your moose console running on localhost:3001.",
            steps: [
              {
                title: "Step 1",
                description:
                  "From your moose application directory link the autogenerated sdks to your global node_modules directory",
                command: "npm link ./moose/<moose-project-name>-sdk",
              },
              {
                title: "Step 2",
                description:
                  "Import the captureEvent function from the sdk in your application",
                command:
                  "import { captureEvent } from '<moose-project-name>-sdk';",
              },
              {
                title: "Step 3",
                description:
                  "Track events in your application using the captureEvent function",
                command: "captureEvent('event-name', { data: 'data' });",
              },
              {
                title: "Step 4",
                description:
                  "Navigate to localhost:3001 to view the events being captured in real-time.",
                action: {
                  label: "Go to localhost:3001",
                  href: "http://localhost:3001",
                },
              },
            ],
          },
          {
            title: "Deployment",
            steps: [
              {
                title: "Deploying Moose",
                description:
                  "Visit the docs to learn more about deploying MooseJS applications.",
                action: {
                  label: "Visit Docs",
                  href: "https://docs.moosejs.com/deploying/summary",
                },
              },
            ],
          },
        ],
      },
      {
        slug: "llm-application",
        title: "LLM Application",
        img: "/images/templates/mjs-img-product-2.svg",
        description:
          "Leverage your custom business data and context to large language models to automate tasks based on data and context.",
        usage: [],
      },
      {
        slug: "data-warehouse",
        title: "Data Warehouse",
        img: "/images/templates/mjs-img-product-3.svg",
        description:
          "Unify data across your business domains, creating a platform optimized for analysis and data-driven strategy.",
        usage: [],
      },
    ],
  };

  const template = content.templateDetails.find(
    (template) => template.slug === params.templateId,
  );

  return (
    <Grid className="h-full w-full relative">
      <div className="col-span-12 md:col-span-6 h-full  pb-5">
        <div className="md:sticky top-20">
          <Section>
            <div>
              <Link href="/templates">
                <Text className="mt-0 pt-5">
                  <span className="text-muted-foreground">Templates / </span>{" "}
                  <span> {template?.title} </span>
                </Text>
              </Link>
            </div>
            <Display>{template?.title}</Display>
            <Text>{template?.description}</Text>
            {template?.cta ? (
              <CTABar>
                <CopyButton
                  copyText={template?.cta?.text ?? "Error_Event"}
                  name={template?.cta?.label ?? "Error_Event"}
                  subject={template?.cta?.text ?? ""}
                >
                  {template?.cta?.label}
                </CopyButton>
              </CTABar>
            ) : (
              <Text className="text-muted-foreground">Coming Soon</Text>
            )}
            <div className="py-10">
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
                      <Separator className="my-5" />
                    </div>
                  )}
                </Grid>
              ))}
            </div>
          </Section>
        </div>
      </div>
      <div className="col-span-12 md:col-span-6  ">
        <div className=" mb-5">
          <Section>
            <div className="bg-muted aspect-[4/3] flex flex-col justify-center">
              <div className="relative h-3/5">
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
            {template?.usage && (
              <TemplateAccordion templateAccordionItems={template.usage} />
            )}
          </Section>
        </div>
      </div>
      <FullWidthContentContainer className="col-span-12 ">
        <FooterSection />
        <EmailSection />
      </FullWidthContentContainer>
    </Grid>
  );
}
