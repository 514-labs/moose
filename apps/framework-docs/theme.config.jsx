import {
  Heading,
  HeadingLevel,
  SmallText,
  SmallTextEmbed,
  textBodyBase,
} from "@/components/typography";
import { cn } from "@/lib/utils";
import Link from "next/link";
import { Python, TypeScript } from "./src/components/language-wrappers";

export default {
  logo: () => (
    <div className="flex flex-row items-center content-center w-[288px]">
      <SmallText className="text-primary">Fiveonefour</SmallText>
      <SmallText className="text-muted-foreground mx-1">Docs</SmallText>
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
    h1: ({ id, children }) => <Heading id={id}>{children}</Heading>,
    h2: ({ id, children }) => (
      <Heading level={HeadingLevel.l2} id={id}>
        {children}
      </Heading>
    ),
    h3: ({ id, children }) => (
      <Heading level={HeadingLevel.l3} id={id}>
        {children}
      </Heading>
    ),
    h4: ({ id, children }) => (
      <Heading level={HeadingLevel.l4} id={id}>
        {children}
      </Heading>
    ),
    p: (props) => {
      return <SmallText {...props} />;
    },
    ul: (props) => (
      <ul className="pl-8 list-disc text-sm leading-7" {...props} />
    ),
    ol: (props) => (
      <ol className="pl-8 list-decimal text-sm leading-7" {...props} />
    ),
    li: (props) => (
      <li className="list-item">
        <span className="text-sm text-primary mb-1 block">
          {props.children}
        </span>
      </li>
    ),
    Python: ({ children }) => <Python>{children}</Python>,
    TypeScript: ({ children }) => <TypeScript>{children}</TypeScript>,
    a: ({ children, href }) => (
      <a href={href} className="text-moose-purple">
        {children}
      </a>
    ),
  },
  color: {
    hue: 220,
    saturation: 0,
  },
  darkMode: true,
  sidebar: {
    defaultMenuCollapseLevel: 1,
    titleComponent({ title, type }) {
      if (type === "separator") {
        return (
          <div className="py-3 mt-2">
            <span className="font-bold text-moose-purple text-base">
              {title}
            </span>
          </div>
        );
      }
      return <SmallText className="text-muted-foreground">{title}</SmallText>;
    },
  },
  toc: {
    title: () => {
      return <SmallTextEmbed> On this page </SmallTextEmbed>;
    },
    headingComponent({ children }) {
      return (
        <SmallText className="my-0 text-muted-foreground font-normal hover:text-primary">
          {children}
        </SmallText>
      );
    },
    extraContent: () => {
      return (
        <div className="border rounded-xl">
          <div className="m-4">
            <SmallText className="my-0 text-xs">
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
  footer: {
    content: (
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
