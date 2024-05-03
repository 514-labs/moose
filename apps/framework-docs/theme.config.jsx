import {
  Display,
  Heading,
  Text,
  TextEmbed,
  HeadingLevel,
  SmallText,
  SmallTextEmbed,
} from "design-system/typography";
import { Logo, Badge } from "design-system/components";
``;

export default {
  logo: () => (
    <div className="flex flex-row">
      <Logo property="moosejs" subProperty="docs" />
      <Badge className="ml-3 mt-1.5" variant={"outline"}>
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
      titleTemplate: "%s – MooseJS",
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
    li: ({ children }) => (
      <li>
        <SmallTextEmbed>{children}</SmallTextEmbed>
      </li>
    ),
    ul: ({ children }) => <SmallTextEmbed>{children}</SmallTextEmbed>,
    ol: ({ children }) => <SmallTextEmbed>{children}</SmallTextEmbed>,
  },
  primaryHue: 220,
  primarySaturation: 0,
  sidebar: {
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
