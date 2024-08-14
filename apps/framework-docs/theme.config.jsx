import {
  Display,
  Heading,
  Text,
  TextEmbed,
  HeadingLevel,
  SmallText,
  SmallTextEmbed,
  textBodyBase,
} from "@514labs/design-system-components/typography";
import { Logo, Badge } from "@514labs/design-system-components/components";
import { cn } from "@514labs/design-system-components/utils";

export default {
  logo: () => (
    <div className="flex flex-row items-center content-center">
      <Logo property="moose" subProperty="docs" className="mr-2" />
      <Badge variant={"outline"} className="w-fit ml-2">
        alpha
      </Badge>
    </div>
  ),
  project: {
    link: "https://github.com/514-labs/moose",
  },
  docsRepositoryBase:
    "https://github.com/514-labs/moose/tree/main/apps/framework-docs",
  useNextSeoProps() {
    return {
      titleTemplate: "%s – Moose",
    };
  },
  head: () => (
    <>
      <link rel="icon" href="/favicon.ico" type="image/x-icon" sizes="16x16" />
    </>
  ),

  components: {
    h1: ({ children }) => <Heading>{children}</Heading>,
    h2: ({ children }) => (
      <Heading longForm level={HeadingLevel.l2}>
        {children}
      </Heading>
    ),
    h3: ({ children }) => (
      <Heading longForm level={HeadingLevel.l3}>
        {children}
      </Heading>
    ),
    h4: ({ children }) => (
      <Heading longForm level={HeadingLevel.l4}>
        {children}
      </Heading>
    ),
    p: ({ children }) => <SmallText>{children}</SmallText>,
    ul: (props) => (
      <ul className={cn("pl-8 list-disc", textBodyBase)} {...props} />
    ),
    ol: (props) => (
      <ol className={cn("pl-8 list-decimal", textBodyBase)} {...props} />
    ),
    li: (props) => (
      <li {...props} className="list-item">
        <SmallTextEmbed className="my-0">{props.children}</SmallTextEmbed>
      </li>
    ),
  },
  primaryHue: 220,
  primarySaturation: 0,
  sidebar: {
    defaultMenuCollapseLevel: 2,
    titleComponent({ title }) {
      return (
        <SmallText className="my-0 text-muted-foreground">{title}</SmallText>
      );
    },
  },
  toc: {
    title: () => {
      return <SmallTextEmbed> On this page </SmallTextEmbed>;
    },
    headingComponent({ children }) {
      return (
        <SmallText className="my-0 text-muted-foreground">{children}</SmallText>
      );
    },
  },
  footer: {
    text: (
      <span>
        MIT | {new Date().getFullYear()} ©{" "}
        <a href="https://fiveonefour.com" target="_blank">
          Fiveonefour Labs Inc
        </a>
        .
      </span>
    ),
  },
};
