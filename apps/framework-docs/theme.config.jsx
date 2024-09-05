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
import Link from "next/link";

export default {
  logo: () => (
    <div className="flex flex-row items-center content-center w-[288px]">
      <Logo property="moose" subProperty="docs" className="mr-2" />
      <Badge variant={"outline"} className="w-fit ml-2">
        alpha
      </Badge>
    </div>
  ),
  project: {
    link: "https://github.com/514-labs/moose",
    icon: (
      <div className="md:w-64">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          width="24"
          height="24"
        >
          <path
            fill="white"
            d="M12.5.75C6.146.75 1 5.896 1 12.25c0 5.089 3.292 9.387 7.863 10.91.575.101.79-.244.79-.546 0-.273-.014-1.178-.014-2.142-2.889.532-3.636-.704-3.866-1.35-.13-.331-.69-1.352-1.18-1.625-.402-.216-.977-.748-.014-.762.906-.014 1.553.834 1.769 1.179 1.035 1.74 2.688 1.25 3.349.948.1-.747.402-1.25.733-1.538-2.559-.287-5.232-1.279-5.232-5.678 0-1.25.445-2.285 1.178-3.09-.115-.288-.517-1.467.115-3.048 0 0 .963-.302 3.163 1.179.92-.259 1.897-.388 2.875-.388.977 0 1.955.13 2.875.388 2.2-1.495 3.162-1.179 3.162-1.179.633 1.581.23 2.76.115 3.048.733.805 1.179 1.825 1.179 3.09 0 4.413-2.688 5.39-5.247 5.678.417.36.776 1.05.776 2.128 0 1.538-.014 2.774-.014 3.162 0 .302.216.662.79.547C20.709 21.637 24 17.324 24 12.25 24 5.896 18.854.75 12.5.75Z"
          ></path>
        </svg>
      </div>
    ),
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
  navigation: {
    prev: true,
    next: true,
  },
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
    a: ({ children }) => (
      <span className="text-pink cursor-pointer">{children}</span>
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
          <div className="flex flex-row justify-start items-center text-muted-foreground gap-2">
            <div className="h-full">
              {(() => {
                switch (title) {
                  case "Get Started":
                    return <Rocket className="h-[20px] w-[20px]" />;
                  case "Develop":
                    return <Code className="h-[20px] w-[20px]" />;
                  case "Deploy":
                    return <Package className="h-[20px] w-[20px]" />;
                  case "Reference":
                    return <Library className="h-[20px] w-[20px]" />;
                  default:
                    return null;
                }
              })()}
            </div>
            <SmallText className="my-0 text-muted-foreground leading-[15px] text-xs">
              {title}
            </SmallText>
          </div>
        );
      }
      return (
        <SmallText className="my-0 text-muted-foreground">{title}</SmallText>
      );
    },
    toggleButton: false,
  },
  toc: {
    title: () => {
      return <SmallTextEmbed> On this page </SmallTextEmbed>;
    },
    headingComponent({ children }) {
      return (
        <SmallText className="my-0 text-muted-foreground font-normal">
          {children}
        </SmallText>
      );
    },
    extraContent: () => {
      return (
        <div className="border rounded-xl">
          <div className="m-4">
            <SmallText className="my-0">
              Have a question or want to provide us with feedback?
            </SmallText>
            <Link href="https://github.com/514-labs/moose/issues/new?title=Feedback%20for%20%E2%80%9CIntroduction%E2%80%9D&labels=feedback">
              <SmallText className="text-pink my-0">Contact us</SmallText>
            </Link>
          </div>
        </div>
      );
    },
  },
  feedback: {
    content: null,
  },
  editLink: {
    component: null,
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
