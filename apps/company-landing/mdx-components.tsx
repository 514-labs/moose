import {
  Display,
  Heading,
  HeadingLevel,
  SuperDisplay,
  Text,
  TextEmbed,
  textBodyBase,
} from "@514labs/design-system/typography";
import { cn } from "@514labs/design-system/utils";
import type { MDXComponents } from "mdx/types";
import Image from "next/image";

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    h1: SuperDisplay,
    h2: Display,
    h3: Heading,
    h4: ({ children }) => (
      <Heading className="font-medium" longForm level={HeadingLevel.l4}>
        {children}
      </Heading>
    ),
    h5: ({ children }) => (
      <Heading className="font-medium" longForm level={HeadingLevel.l5}>
        {children}
      </Heading>
    ),
    ul: (props) => (
      <ul className={cn("pl-8 list-disc", textBodyBase)} {...props} />
    ),
    ol: (props) => (
      <ol className={cn("pl-8 list-decimal", textBodyBase)} {...props} />
    ),
    li: (props) => (
      <li {...props} className="list-item">
        <TextEmbed className="my-0">{props.children}</TextEmbed>
      </li>
    ),
    a: (props) => <a {...props} className="text-primary underline" />,
    p: Text,
    Image: (props) => <Image {...props} />,
    ...components,
  };
}
