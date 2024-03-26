import { Display, Heading, SuperDisplay, Text } from "design-system/typography";
import type { MDXComponents } from "mdx/types";

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    h1: SuperDisplay,
    h2: Display,
    h3: Heading,
    p: Text,
    ...components,
  };
}
