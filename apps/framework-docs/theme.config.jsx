import {
  Display,
  Heading,
  Text,
  TextEmbed,
  HeadingLevel,
  GradientText,
  SmallText,
  SmallTextEmbed,
  textBodyBase,
} from "@514labs/design-system-components/typography";
import { Logo, Badge } from "@514labs/design-system-components/components";
import { cn } from "@514labs/design-system-components/utils";
import { Code, Rocket, Package, Library } from "lucide-react";

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
        <SmallTextEmbed className="mb-1">{props.children}</SmallTextEmbed>
      </li>
    ),
  },
  primaryHue: 220,
  primarySaturation: 0,
  sidebar: {
    defaultMenuCollapseLevel: 1,
    titleComponent({ title, type }) {
      if (type === "separator") {
        return (
          <div className="flex flex-row justify-start items-center text-accent-foreground gap-3">
            <div className="bg-muted rounded-md h-full">
              {(() => {
                switch (title) {
                  case "Get Started":
                    return <Rocket className="m-2 h-[20px] w-[20px]" />;
                  case "Develop":
                    return <Code className="m-2 h-[20px] w-[20px]" />;
                  case "Deploy":
                    return <Package className="m-2 h-[20px] w-[20px]" />;
                  case "Reference":
                    return <Library className="m-2 h-[20px] w-[20px]" />;
                  default:
                    return null;
                }
              })()}
            </div>
            <SmallText className="my-0">{title}</SmallText>
          </div>
        );
      }
      return (
        <SmallText className="my-0 text-muted-foreground">{title}</SmallText>
      );
    },
    toggleButton: true,
  },
  toc: {
    title: () => {
      return <SmallTextEmbed> On this page </SmallTextEmbed>;
    },
    headingComponent({ children }) {
      return (
        <SmallText className="my-0 text-muted-foreground prose">
          {children}
        </SmallText>
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
