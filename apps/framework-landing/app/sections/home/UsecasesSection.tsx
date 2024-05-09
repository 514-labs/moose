import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "design-system/components";
import {
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";

import {
  Heading,
  HeadingLevel,
  SmallText,
  Text,
} from "design-system/typography";

const content = {
  title: "Purpose built for data focused applications",
  description:
    "Moose provides the primitives you need for batch, streaming and event-driven data applications and lets you focus on your business logic. ",
  disclaimer:
    "If you're building a user workflow centric, transactional application, we recomend you'll need to use a more general purpose framework like Next.js, Rails, Django, or Express.js.",
  usecases: [
    {
      title: "Data-centric Generative AI Applications",
      description:
        "Build data aware LLM + RAG applications to surface insights for your users",
      details: [
        "richly integrate your data with your applications: as your data changes, your applications change with them",
        "make your data available to developers across your company: your data, your data model, their applications",
        "automate good data consumption practices: data models, SDK and API definition, usage tracking, versioning",
        "wire your data flows into your applications, including your large language models (write your RAG where you write your app)",
      ],
    },
    {
      title: "Prediction, Automation, Operations",
      description:
        "Streamline digital operations with predictions backed by high quality data",
      details: [
        "Enrich your data streams with whatever sources you need, to make self-evidently valuable data",
        "Build applications (for understanding, observability, visualization, automation and more) that use your data, or let your users build them themselves",
      ],
    },
    {
      title: "Internal BI and Analytics Applications",
      description:
        "Understand users and business operations across technologies and products",
      details: [
        "Extract untapped insights from idle data by transforming your data swamp into a easy-to-consume metrics APIs",
        "Make your data products as usable as possible: each data product has its own data model, versioning (and migration), automated API and SDK definition, usage statistics",
        "Use your data infrastructure or use our open source infrastructure out of the box",
      ],
    },
    {
      title: "End user facing analytics and applications",
      description:
        "Embed interactive visualizations and real-time data feeds into your product",
      details: [
        "richly integrate your data with your applications: as your data changes, your applications change with them",
        "make your data available to developers across your company: your data, your data model, their applications",
        "automate good data consumption practices: data models, SDK and API definition, usage tracking, versioning",
        "wire your data flows into your applications, including your large language models (write your RAG where you write your app)",
      ],
    },
  ],
};

const UsecasesSectionInfo = () => {};

const UsecasesAccordion = () => {
  return (
    <div>
      <Accordion type="single" collapsible>
        {content.usecases.map((usecase, index) => {
          return (
            <AccordionItem
              key={index}
              value={`item-${index}`}
              className="border-none"
            >
              <AccordionTrigger>
                <div className="flex flex-col text-start justify-start">
                  <Text className="my-0">{usecase.title}</Text>
                  <Text className="my-0 text-muted-foreground">
                    {usecase.description}
                  </Text>
                </div>
              </AccordionTrigger>
              <AccordionContent>
                <ul className="border rounded-xl px-5">
                  {usecase.details.map((detail, index) => {
                    return (
                      <li key={index}>
                        <Text className="text-muted-foreground">{detail}</Text>
                      </li>
                    );
                  })}
                </ul>
              </AccordionContent>
            </AccordionItem>
          );
        })}
      </Accordion>
      <div className="my-5 text-muted-foreground text-xs leading border p-5 rounded-xl">
        {content.disclaimer}
      </div>
    </div>
  );
};

export const UsecasesSection = () => {
  return (
    <>
      <Section className="mb-0">
        <Grid className="mb-12 2xl:mb-20">
          <HalfWidthContentContainer>
            <Heading>{content.title}</Heading>
          </HalfWidthContentContainer>
        </Grid>

        <Grid>
          <HalfWidthContentContainer>
            <Text className="text-muted-foreground">{content.description}</Text>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer>
            <UsecasesAccordion />
          </HalfWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
