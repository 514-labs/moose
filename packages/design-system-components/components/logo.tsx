import { GradientText, Text } from "./typography/standard";
import { cn } from "../lib/utils";

export const Logo = ({
  property,
  subProperty,
  className,
}: {
  property: string;
  subProperty?: string;
  className?: string;
}) => {
  return (
    <>
      <GradientText className={cn("my-0", className)}>{property}</GradientText>
      {subProperty && (
        <Text className="my-0 ml-2 text-muted-foreground">{subProperty}</Text>
      )}
    </>
  );
};
