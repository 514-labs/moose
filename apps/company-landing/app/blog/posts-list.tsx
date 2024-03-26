import {
  Section,
  Grid,
  HalfWidthContentContainer,
} from "design-system/components/containers";
import { type Post } from "../../lib/posts";
import Link from "next/link";
import { Heading, Text } from "design-system/typography";

export function Posts({ posts }: { posts: Post[] }) {
  return (
    <Section>
      <Grid>
        {posts.map(({ slug, title, description, publishedAt, categories }) => (
          <>
            <HalfWidthContentContainer>
              <Text className="mb-0">
                {new Date(publishedAt).toLocaleDateString(undefined, {
                  weekday: "short",
                  year: "numeric",
                  month: "long",
                  day: "numeric",
                })}
              </Text>
              <Text className="mt-0">
                {categories.map((cat, i) => `${i ? ", " : ""}${cat}`)}
              </Text>
            </HalfWidthContentContainer>
            <HalfWidthContentContainer>
              <Link href={`blog/${slug}`}>
                <Heading>{title}</Heading>
                <Text>{description}</Text>
              </Link>
            </HalfWidthContentContainer>
          </>
        ))}
      </Grid>
    </Section>
  );
}
