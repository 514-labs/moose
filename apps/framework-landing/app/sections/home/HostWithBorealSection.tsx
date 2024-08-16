import Link from "next/link";
import {
  FullWidthContentContainer,
  Section,
} from "@514labs/design-system-components/components/containers";
import { CTABar } from "../../page";
import {
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { TrackCtaButton } from "../../trackable-components";
import React from "react";

export const SecondaryCTASection = () => {
  const content = {
    title: "Host your Moose projects with Bóreal",
    description: "No need to configure or secure any infrastructure.",
    ctas: [
      {
        href: "https://boreal.cloud/sign-up",
        action: "",
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
      <div className="relative rounded-3xl p-[3px]  bg-gradient-to-b from-green-400 to-black ">
        <div className="bg-background rounded-3xl z-10">
          <FullWidthContentContainer className="rounded-3xl">
            <div className="backdrop-brightness-50 backdrop-blur-md w-full f-full flex flex-row align-middle justify-between p-10 rounded-3xl">
              <div className="flex flex-col gap-1">
                <Heading className="my-0" level={HeadingLevel.l2}>
                  {content.title}
                </Heading>
                <Heading
                  className="my-0 text-muted-foreground"
                  level={HeadingLevel.l4}
                >
                  {content.description}
                </Heading>
              </div>
              <div className="flex flex-col align-middle justify-center">
                <CTABar>
                  {content.ctas.map((cta, index) => (
                    <Link key={index} href={cta.href}>
                      <TrackCtaButton
                        name={`Boreal CTA: ${content.title}`}
                        subject={`${cta.label} - ${cta.variant}`}
                        targetUrl={cta.href}
                        variant={cta.variant as "default" | "outline"}
                      >
                        {cta.label}
                      </TrackCtaButton>
                    </Link>
                  ))}
                </CTABar>
              </div>
            </div>
          </FullWidthContentContainer>
        </div>
      </div>
    </Section>
  );
};
