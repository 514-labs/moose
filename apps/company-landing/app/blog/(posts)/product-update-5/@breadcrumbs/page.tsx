import { Text } from "@514labs/design-system-components/typography";

const getMetaFromParent = async () => {
  const { metadata } = await import(`../page.mdx`);
  return metadata;
};

export default async function Breadcrumbs() {
  const meta = await getMetaFromParent();

  return (
    <div>
      <Text className="mb-0">
        <a href="/blog" className=" text-muted-foreground">
          Blog /
        </a>{" "}
        <a href="">{meta.title}</a>
      </Text>
    </div>
  );
}
