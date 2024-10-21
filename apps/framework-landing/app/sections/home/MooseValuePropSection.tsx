import {
  Section,
  Grid,
  ThirdWidthContentContainer,
} from "@514labs/design-system-components/components/containers";
import {
  Heading,
  HeadingLevel,
} from "@514labs/design-system-components/typography";
import { IconCard } from "@514labs/design-system-components/components";
import { Bot, Cloud, Wand2 } from "lucide-react";

export default function MooseValuePropSection() {
  return (
    <Section className="max-w-5xl mx-auto">
      <Heading
        level={HeadingLevel.l2}
        className="max-w-5xl justify-center align-center text-center md:mb-24 sm:text-5xl"
      >
        Tools you love, context you need.
        <span className="bg-gradient bg-clip-text text-transparent">
          {" "}
          Built for modern, AI-centric development workflows
        </span>
      </Heading>
      <Grid>
        <ThirdWidthContentContainer>
          <IconCard
            title="Local to cloud development workflows"
            description="Build your app locally using the tools you love then take your app to the cloud reliably"
            Icon={Cloud}
          />
        </ThirdWidthContentContainer>
        <ThirdWidthContentContainer>
          <IconCard
            title="Data Model awareness for you and your AI co-pilots"
            description="Data context pulled directly into your development workflow at dev and build time"
            Icon={Wand2}
          />
        </ThirdWidthContentContainer>
        <ThirdWidthContentContainer>
          <IconCard
            title="Infra automation to keep you focused"
            description="Write code, get infra. Moose uses framework-defined infra to keep you focused"
            Icon={Bot}
          />
        </ThirdWidthContentContainer>
      </Grid>
    </Section>
  );
}
