"use client";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@514labs/design-system-components/components";
import { Heading, Text } from "@514labs/design-system-components/typography";
import Diagram from "../../spline";
import { useRef } from "react";
const content = {
  title: "Analytics or User Facing Application",
  description:
    "Deliver structured data and insights to user facing applications, AI/ML models, analyst notebooks, or enterprise BI software.",
  usecases: [
    {
      title: "Data Application Logic",
      layer: "TOP",
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
      title: "Moose Defined Infrastructure",
      layer: "MIDDLE",
      description:
        "Understand users and business operations across technologies and products",
      details: [
        "Extract untapped insights from idle data by transforming your data swamp into a easy-to-consume metrics APIs",
        "Make your data products as usable as possible: each data product has its own data model, versioning (and migration), automated API and SDK definition, usage statistics",
        "Use your data infrastructure or use our open source infrastructure out of the box",
      ],
    },
    {
      title: "Foundation Infrastructure",
      layer: "BOTTOM",
      description:
        "Streamline digital operations with predictions backed by high quality data",
      details: [
        "Enrich your data streams with whatever sources you need, to make self-evidently valuable data",
        "Build applications (for understanding, observability, visualization, automation and more) that use your data, or let your users build them themselves",
      ],
    },
  ],
};

const DiagramAccordion = ({ spline }: { spline: any }) => {
  return (
    <div>
      <Accordion type="single" collapsible>
        {content.usecases.map((usecase, index) => {
          return (
            <AccordionItem
              key={index}
              value={`item-${index}`}
              className="last:border-none"
              onClick={() => {
                spline.current.emitEvent("mousedown", usecase.layer);
              }}
            >
              <AccordionTrigger>
                <div className="flex flex-col text-start justify-start">
                  <Text className="my-0">{usecase.title}</Text>
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
    </div>
  );
};

export default function DiagramSection() {
  const spline = useRef();
  return (
    <div className="flex h-[600px]">
      <div className="w-1/2">
        <Diagram spline={spline} />
      </div>
      <div className="w-1/2">
        <DiagramAccordion spline={spline} />
      </div>
    </div>
  );
}
