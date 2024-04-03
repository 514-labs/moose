import { Posts } from "./posts-list";
import { getPosts } from "../../lib/posts";
import { Section } from "design-system/components/containers";
import { Display } from "design-system/typography";
import { sendServerEvent } from "event-capture/server-event";

export default async function Blog() {
  sendServerEvent("page_view", { page: "blog" });
  const posts = await getPosts();

  return (
    <Section>
      <Display>Blog</Display>
      <Posts posts={posts} />
    </Section>
  );
}
