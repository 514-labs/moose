import type { MDXComponents } from "mdx/types";

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    // This allows you to provide custom React components
    // to be used in MDX files. You can import and use any
    // React component you want, including inline styles,
    // components from other libraries, and more.
    ...components,
  };
}
