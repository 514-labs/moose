import {
  Section,
  Grid,
  FullWidthContentContainer,
  ThirdWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { Heading, Text } from "@514labs/design-system/typography";

export const FeaturesSection = () => {
  const content = {
    title: "Everything you need to build data-driven experiences",
    features: [
      {
        title: "Your tools, your way",
        description:
          "Git-based workflows, languages you know, and the IDE of your choice. Moose is built to slot into your existing tools from your local env, to CI/CD and the cloud",
      },
      {
        title: "Auto migrations",
        description:
          "Moose automatically manages schema evolution across your data stack, so you don’t break upstream and downstream dependencies",
      },
      {
        title: "End-to-end typed",
        description:
          "Moose types all your components from data capture through consumption, so you catch issues at build time, instead of in production",
      },
      {
        title: "Built-in testing",
        description:
          "With Moose’s built-in testing framework, you can manage sample data and automatically test data pipelines as you’re developing",
      },
      {
        title: "Data product APIs",
        description:
          "Moose automatically creates API endpoints for your data products, and generates SDKs in the languages of your choice (typescript, python, java, rust, etc).",
      },
      {
        title: "Local dev server",
        description:
          "Run your whole data/analytics stack locally: see the impact of changes in real time as you edit code - just like developing a web application",
      },
      {
        title: "Built-in testing",
        description:
          "With Moose’s built-in testing framework, you can manage sample data and automatically test data pipelines as you’re developing",
      },
      {
        title: "Data product APIs",
        description:
          "Moose automatically creates API endpoints for your data products, and generates SDKs in the languages of your choice (typescript, python, java, rust, etc).",
      },
      {
        title: "Local dev server",
        description:
          "Run your whole data/analytics stack locally: see the impact of changes in real time as you edit code - just like developing a web application",
      },
    ],
  };

  return (
    <>
      <Section>
        <Grid>
          <FullWidthContentContainer>
            <Heading> {content.title} </Heading>
          </FullWidthContentContainer>
        </Grid>
      </Section>
      <Section>
        <Grid>
          {content.features.map((feature, index) => {
            return (
              <ThirdWidthContentContainer key={index}>
                <Text className="my-0">{feature.title}</Text>
                <Text className="my-0 text-muted-foreground">
                  {feature.description}
                </Text>
              </ThirdWidthContentContainer>
            );
          })}
        </Grid>
      </Section>
    </>
  );
};
