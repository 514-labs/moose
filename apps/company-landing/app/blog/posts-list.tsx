import {
  Section,
  Grid,
  HalfWidthContentContainer,
} from "design-system/components/containers";
import { type Post } from "../../lib/posts";
import Link from "next/link";
import { Heading, Text } from "design-system/typography";
import { humanReadableDate } from "../../lib/formatter";

export function Posts({ posts }: { posts: Post[] }) {
  return (
    <Section>
      <Grid>
        {posts.map(({ slug, title, description, publishedAt, categories }) => (
          <>
            <HalfWidthContentContainer>
              <Text className="mb-0">{humanReadableDate(publishedAt)}</Text>
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
