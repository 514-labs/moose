import { Posts } from "./posts-list";
import { getPosts } from "../../lib/posts";
import { Section } from "@514labs/design-system/components/containers";
import { Display } from "@514labs/design-system/typography";

export default async function Blog() {
  const posts = await getPosts();

  return (
    <Section>
      <Display>Blog</Display>
      <Posts posts={posts} />
    </Section>
  );
}
