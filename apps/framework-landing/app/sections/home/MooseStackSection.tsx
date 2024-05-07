import {
  FullWidthContentContainer,
  Grid,
  HalfWidthContentContainer,
  Section,
} from "design-system/components/containers";
import { Heading, Text } from "design-system/typography";
import Image from "next/image";

export const MooseStackSection = () => {
  const content = {
    title: "The Moose Stack",
    stack: [
      {
        title: "Data Experience (React or BI apps)",
        description:
          "Integrate Moose UI components into rich react based applications, connect your existing frontend applications",
      },
      {
        title: "Data service (built with Moose)",
        description:
          "Build everything you need to support your data intensive application. From data capture to repeatable insights",
      },
      {
        title: "Data Infrastructure (provisioned by Moose)",
        description:
          "Moose can be configured to run on your favorite infrastructure providers to support your use cases. From small to large scale",
      },
    ],
  };

  return (
    <>
      <Section>
        <Grid>
          <FullWidthContentContainer>
            <Heading>{content.title}</Heading>
          </FullWidthContentContainer>
        </Grid>
      </Section>
      <Section>
        <Grid className="gap-y-5">
          <HalfWidthContentContainer className="sticky md:top-24 flex items-center justify-center">
            <div className="relative w-full aspect-video">
              <Image
                priority
                className="hidden dark:block"
                src="/images/how-it-works/IMG-STACK-DARK.svg"
                fill
                alt="man in jacket"
                sizes="(max-width: 768px) 150vw, 25vw"
              />
              <Image
                priority
                className="block dark:hidden"
                src="/images/how-it-works/IMG-STACK-LIGHT.svg"
                fill
                alt="man in jacket"
                sizes="(max-width: 768px) 150vw, 25vw"
              />
            </div>
          </HalfWidthContentContainer>
          <HalfWidthContentContainer className="flex flex-col justify-center">
            <div className="flex flex-col gap-5">
              {content.stack.map((step, index) => {
                return (
                  <div className="flex flex-row gap-5" key={index}>
                    <div>
                      <Text className="my-0 text-muted-foreground">
                        {`0${index + 1}`}
                      </Text>
                    </div>
                    <div>
                      <Text className="my-0">{step.title}</Text>
                      <Text className="my-0 text-muted-foreground">
                        {step.description}
                      </Text>
                    </div>
                  </div>
                );
              })}
            </div>
          </HalfWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
};
