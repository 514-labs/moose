import { Heading, Text } from "design-system/typography";
import { humanReadableDate } from "../../../../../lib/formatter";

const getMetaFromParent = async () => {
  const { metadata } = await import(`../page.mdx`);
  return metadata;
};

export default async function Meta() {
  const meta = await getMetaFromParent();

  return (
    <div>
      <Heading>{meta.title}</Heading>
      <Text>{humanReadableDate(meta.publishedAt)}</Text>
      {meta.categories.map((cat, i) => (
        <Text key={i}>{cat}</Text>
      ))}
    </div>
  );
}
