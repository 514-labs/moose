import {
  Display,
  Heading,
  HeadingLevel,
  SuperDisplay,
  Text,
  TextEmbed,
} from "design-system/typography";
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
    ul: (props) => (
      <ul
        className="pl-8 list-disc text-primary leading-normal 2xl:leading-normal text-lg sm:text-xl 2xl:text-2xl 3xl:text-3xl"
        {...props}
      />
    ),
    ol: (props) => (
      <ol
        className="pl-8 list-decimal text-primary leading-normal 2xl:leading-normal text-lg sm:text-xl 2xl:text-2xl 3xl:text-3xl"
        {...props}
      />
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
