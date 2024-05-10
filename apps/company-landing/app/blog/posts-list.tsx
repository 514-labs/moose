import {
  Section,
  Grid,
  HalfWidthContentContainer,
} from "@514labs/design-system/components/containers";
import { type Post } from "../../lib/posts";
import Link from "next/link";
import { Heading, Text } from "@514labs/design-system/typography";
import { humanReadableDate } from "../../lib/formatter";

export function Posts({ posts }: { posts: Post[] }) {
  return (
    <Section className="px-0">
      <Grid>
        {posts.map(({ slug, title, description, publishedAt, categories }) => (
          <>
            <HalfWidthContentContainer>
              <Text className="mb-0">
                {"Published " + humanReadableDate(publishedAt)}
              </Text>
              <Text className="mt-0 text-muted-foreground">
                {categories.map((cat, i) => `${i ? ", " : ""}${cat}`)}
              </Text>
            </HalfWidthContentContainer>
            <HalfWidthContentContainer>
              <Link href={`blog/${slug}`}>
                <Heading>{title}</Heading>
                <Text>
                  {description} (<span className=" underline">more</span>)
                </Text>
              </Link>
            </HalfWidthContentContainer>
          </>
        ))}
      </Grid>
    </Section>
  );
}
