import { Text } from "design-system/typography";
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
      <Text className="mt-0">
        {meta.categories.map((cat: string, i: number) => (
          <span key={i}>{cat}</span>
        ))}
      </Text>
    </div>
  );
}
