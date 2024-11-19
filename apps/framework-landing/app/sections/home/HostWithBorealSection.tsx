import Link from "next/link";
import { Section } from "@514labs/design-system-components/components/containers";
import {
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { TrackButton } from "@514labs/design-system-components/trackable-components";
import React from "react";

export const SecondaryCTASection = () => {
  const content = {
    title: "Host your Moose projects with Boréal",
    description: "No need to configure or secure any infrastructure.",
    ctas: [
      {
        href: "https://boreal.cloud/sign-up",
        action: "boreal-sign-up",
        label: "Deploy Now",
        variant: "default",
      },
      {
        href: "https://boreal.cloud",
        action: "cta-early-access",
        label: "Learn More",
        variant: "outline",
      },
    ],
  };

  return (
    <Section className="mx-auto max-w-5xl">
      <div className="rounded-2xl border-2 flex flex-col md:flex-row justify-between items-center md:items-start gap-10 p-8">
        <Heading
          className="w-full md:w-1/2 mt-0 text-center md:text-start"
          level={HeadingLevel.l2}
        >
          {content.title}
        </Heading>
        <div className="flex-grow flex flex-col w-full gap-4">
          {content.ctas.map((cta, index) => (
            <Link key={index} href={cta.href}>
              <TrackButton
                name={`Boreal CTA: ${content.title}`}
                subject={cta.label}
                targetUrl={cta.href}
                variant={cta.variant as "default" | "outline"}
                size="lg"
                className="w-full"
              >
                {cta.label}
              </TrackButton>
            </Link>
          ))}
        </div>
      </div>
    </Section>
  );
};
