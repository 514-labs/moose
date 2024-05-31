import BlogMeta from "../../../blog-meta";

const getMetaFromParent = async () => {
  const { metadata } = await import(`../page.mdx`);
  return metadata;
};

export default async function Meta() {
  const meta = await getMetaFromParent();

  return <BlogMeta meta={meta} />;
}
