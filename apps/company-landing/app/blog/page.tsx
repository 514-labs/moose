import { Section } from "design-system/components/containers";
import { Posts } from "./posts-list";
import { getPosts } from "../../lib/posts";
import { Display } from "design-system/typography";
import FooterSection from "../sections/FooterSection";
import { EmailSection } from "../sections/EmailSection";

export default async function Blog() {
  const posts = await getPosts();

  return (
    <>
      <Section>
        <Display>Blog</Display>
        <Posts posts={posts} />
      </Section>
      <FooterSection />
      <EmailSection />
    </>
  );
}
