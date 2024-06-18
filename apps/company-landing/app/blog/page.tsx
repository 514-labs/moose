import { Posts } from "./posts-list";
import { getPosts } from "../../lib/posts";
import { Section } from "@514labs/design-system-components/components/containers";
import FooterSection from "../sections/FooterSection";
import { Display } from "@514labs/design-system-components/typography";

export default async function Blog() {
  const posts = await getPosts();

  return (
    <Section className="px-6 w-full relative mx-auto xl:max-w-screen-xl">
      <Display>Blog</Display>
      <Posts posts={posts} />
      <FooterSection />
    </Section>
  );
}
