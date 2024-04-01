import { Text } from "./typography/standard";

export const Logo = ({
  property,
  subProperty,
}: {
  property: string;
  subProperty?: string;
}) => {
  return (
    <>
      <Text className="my-0">{property}</Text>
      {subProperty && (
        <Text className="my-0 ml-2 text-muted-foreground">{subProperty}</Text>
      )}
    </>
  );
};
