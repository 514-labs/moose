import {
  Heading,
  HeadingLevel,
  SmallText,
  Text,
} from "@/components/typography";

import { cn } from "@/lib/utils";
import Image from "next/image";
import Link from "next/link";
import { Python, TypeScript } from "./src/components/language-wrappers";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";

// Base text styles that match your typography components
const baseTextStyles = {
  small:
    "text-primary text-sm sm:text-sm 2xl:text-base 3xl:text-lg leading-normal",
  regular:
    "text-primary text-base sm:text-lg 2xl:text-xl 3xl:text-2xl leading-normal",
  heading: "text-primary font-semibold",
};

export function Logo() {
  return (
    <Link href="https://www.fiveonefour.com" className="shrink-0">
      <Image
        src="/logo.png"
        alt="logo"
        width={48}
        height={48}
        priority
        className="hidden dark:block"
      />
      <Image
        src="/images/logo-light.png"
        alt="logo"
        width={48}
        height={48}
        priority
        className="block dark:hidden"
      />
    </Link>
  );
}

export function LogoBreadcrumb() {
  return (
    <Breadcrumb>
      <BreadcrumbList className="flex-nowrap">
        <BreadcrumbItem key="company">
          <BreadcrumbLink
            href="https://www.fiveonefour.com"
            className={baseTextStyles.small}
          >
            Fiveonefour
          </BreadcrumbLink>
        </BreadcrumbItem>
        <BreadcrumbSeparator />
        <BreadcrumbItem key="docs">
          <BreadcrumbLink
            href="/"
            className={cn(baseTextStyles.small, "text-muted-foreground")}
          >
            Docs
          </BreadcrumbLink>
        </BreadcrumbItem>
      </BreadcrumbList>
    </Breadcrumb>
  );
}

export default {
  logo: () => (
    <div className="flex items-center gap-2">
      <Logo />
      <LogoBreadcrumb />
    </div>
  ),
  logoLink: false,
  project: {
    link: "https://github.com/514-labs/moose",
  },
  docsRepositoryBase:
    "https://github.com/514-labs/moose/tree/main/apps/framework-docs",
  head: () => (
    <>
      <link rel="icon" href="/favicon.ico" type="image/x-icon" sizes="16x16" />
    </>
  ),
  navbar: {
    extraContent: () => (
      <Link href="https://www.boreal.cloud/sign-in">
        <Button variant="default">Sign In</Button>
      </Link>
    ),
  },
  navigation: {
    prev: true,
    next: true,
  },
  components: {
    // Heading components with stable rendering
    h1: ({ children, ...props }) => (
      <Heading {...props} level={HeadingLevel.l1}>
        {children}
      </Heading>
    ),
    h2: ({ children, ...props }) => (
      <Heading {...props} level={HeadingLevel.l2}>
        {children}
      </Heading>
    ),
    h3: ({ children, ...props }) => (
      <Heading {...props} level={HeadingLevel.l3}>
        {children}
      </Heading>
    ),
    h4: ({ children, ...props }) => (
      <Heading {...props} level={HeadingLevel.l4}>
        {children}
      </Heading>
    ),
    // Text components with direct styling
    p: ({ children, className, ...props }) => (
      <p className={cn("my-5", baseTextStyles.small, className)} {...props}>
        {children}
      </p>
    ),
    // List components with consistent styling
    ul: ({ children, className, ...props }) => (
      <ul
        className={cn(
          "pl-8 list-disc leading-7",
          baseTextStyles.small,
          className,
        )}
        {...props}
      >
        {children}
      </ul>
    ),
    ol: ({ children, className, ...props }) => (
      <ol
        className={cn(
          "pl-8 list-decimal leading-7",
          baseTextStyles.small,
          className,
        )}
        {...props}
      >
        {children}
      </ol>
    ),
    li: ({ children }) => (
      <li className={cn("list-item list-disc my-3", baseTextStyles.small)}>
        {children}
      </li>
    ),
    // Language-specific components
    Python: ({ children, ...props }) => <Python {...props}>{children}</Python>,
    TypeScript: ({ children, ...props }) => (
      <TypeScript {...props}>{children}</TypeScript>
    ),
    // Link styling
    a: ({ children, href, className }) => (
      <a
        href={href}
        className={cn(
          "text-moose-purple hover:text-moose-purple/90 transition-colors",
          className,
        )}
      >
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
  },
  footer: {
    content: () => {
      const year = new Date().getFullYear();
      return (
        <p className={baseTextStyles.small}>
          MIT | {year} ©{" "}
          <a
            href="https://fiveonefour.com"
            target="_blank"
            rel="noopener noreferrer"
            className="text-moose-purple hover:text-moose-purple/90 transition-colors"
          >
            Fiveonefour Labs Inc
          </a>
        </p>
      );
    },
  },
};
