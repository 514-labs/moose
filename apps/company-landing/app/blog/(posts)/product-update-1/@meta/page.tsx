import { Text } from "design-system/typography";
import { Avatar } from "design-system/avatar";
import { humanReadableDate } from "../../../../../lib/formatter";

const getMetaFromParent = async () => {
  const { metadata } = await import(`../page.mdx`);
  return metadata;
};

export default async function Meta() {
  const meta = await getMetaFromParent();

  return (
    <div>
      <Text className="mb-0">{humanReadableDate(meta.publishedAt)}</Text>
      {meta.categories.map((cat: string, i: number) => {
        if (i === meta.categories.length - 1) {
          return (
            <Text className="mt-0" key={i}>
              {cat}
            </Text>
          );
        } else {
          return (
            <Text className="mt-0" key={i}>
              {cat},{" "}
            </Text>
          );
        }
      })}
    </div>
  );
}
