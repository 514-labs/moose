import {
  Section,
  Grid,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { IconCard } from "@514labs/design-system-components/components";
import { ServerIcon } from "lucide-react";

export default function MooseValuePropSection() {
  return (
    <>
      <Section className="2xl:max-w-6xl mx-auto flex flex-col items-center px-5 my-16 sm:my-64">
        <Grid>
          <FullWidthContentContainer>
            <Heading
              level={HeadingLevel.l2}
              className="max-w-5xl justify-center align-center text-center md:mb-24 sm:text-5xl"
            >
              Designed with the future of software development in mind.
              <span className="bg-gradient bg-clip-text text-transparent">
                {" "}
                All in pure TypeScript or Python.
              </span>
            </Heading>
          </FullWidthContentContainer>
        </Grid>
      </Section>
      <Section>
        <Grid>
          <ThirdWidthContentContainer>
            <IconCard
              title="Simple Primitives"
              description="Simple data engineering abstractions to help you do more with less boilerplate"
              Icon={ServerIcon}
            />
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer>
            <IconCard
              title="Automated Glue"
              description="Write code, get the results you expect. Moose makes sure the data keeps flowing"
              Icon={ServerIcon}
            />
          </ThirdWidthContentContainer>
          <ThirdWidthContentContainer>
            <IconCard
              title="Build-time Data Context"
              description="Never guess a table name or field. Moose pulls data context into dev workflows"
              Icon={ServerIcon}
            />
          </ThirdWidthContentContainer>
        </Grid>
      </Section>
    </>
  );
}
