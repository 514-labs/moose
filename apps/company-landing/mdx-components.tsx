import {
  Display,
  Heading,
  HeadingLevel,
  SuperDisplay,
  Text,
  TextEmbed,
  textBodyBase,
} from "@514labs/design-system-components/typography";
import { cn } from "@514labs/design-system-components/utils";
import type { MDXComponents } from "mdx/types";
import Image from "next/image";

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    h1: ({ children, id }) => <SuperDisplay id={id}>{children}</SuperDisplay>,
    h2: ({ children, id }) => <Display id={id}>{children}</Display>,
    h3: ({ children, id }) => <Heading id={id}>{children}</Heading>,
    h4: ({ id, children }) => (
      <Heading className="font-medium" longForm level={HeadingLevel.l4} id={id}>
        {children}
      </Heading>
    ),
    h5: ({ id, children }) => (
      <Heading className="font-medium" longForm level={HeadingLevel.l5} id={id}>
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
    pre: (props) => (
      <pre {...props} className="bg-muted rounded-md p-2 overflow-x-scroll" />
    ),
    code: (props) => (
      <code {...props} className="bg-muted p-1 rounded-md break-words" />
    ),
    a: (props) => <a {...props} className="text-primary underline" />,
    p: Text,
    Image: (props) => <Image {...props} />,
    ...components,
  };
}
