import { GradientText, SmallText, Text } from "./typography/standard";
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
        <SmallText className="my-0 ml-2 text-muted-foreground border rounded-full px-2 py-0.5">
          {subProperty}
        </SmallText>
      )}
    </>
  );
};
