import {
  Grid,
  ThirdWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { Text } from "@514labs/design-system/typography";

interface Feature {
  title: string;
  description: string;
}

export const FeatureGrid = ({ features }: { features: Feature[] }) => {
  return (
    <Grid>
      {features.map((feature, index) => {
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
  );
};
