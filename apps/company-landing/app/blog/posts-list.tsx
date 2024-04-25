import {
  Section,
  Grid,
  HalfWidthContentContainer,
} from "design-system/components/containers";
import React from "react";
import { type Post } from "../../lib/posts";
import Link from "next/link";
import { Heading, Text } from "design-system/typography";
import { humanReadableDate } from "../../lib/formatter";
import FooterSection from "../sections/FooterSection";
import { EmailSection } from "../sections/EmailSection";

export function Posts({ posts }: { posts: Post[] }) {
  return (
    <>
      <Section className="px-0">
        <Grid>
          {posts.map(
            ({ slug, title, description, publishedAt, categories }) => (
              <React.Fragment key={slug}>
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
              </React.Fragment>
            ),
          )}
        </Grid>
      </Section>
    </>
  );
}
