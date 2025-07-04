import { Heading, HeadingLevel } from "@/components/typography";

import { cn } from "@/lib/utils";
import Image from "next/image";
import Link from "next/link";
import { Python, TypeScript } from "./src/components/language-wrappers";
import { ImageZoom } from "nextra/components";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { useRouter } from "next/router";
import { useConfig } from "nextra-theme-docs";
import { PathConfig } from "./src/components/ctas";
import Script from "next/script";

// Base text styles that match your typography components
const baseTextStyles = {
  small:
    "text-muted-foreground text-sm sm:text-sm 2xl:text-base 3xl:text-md leading-normal",
  regular:
    "text-primary text-base sm:text-lg 2xl:text-xl 3xl:text-2xl leading-normal",
  heading: "text-primary font-semibold",
};

export function Logo() {
  return (
    <Link
      href="https://www.fiveonefour.com"
      className="shrink-0 flex items-center"
    >
      <div className="w-[16px] h-[16px] mr-2 relative">
        <Image
          src="/logo-light.png"
          alt="logo"
          fill
          sizes="16px"
          priority
          className="object-contain object-center hidden dark:block"
        />
        <Image
          src="/logo-dark.png"
          alt="logo"
          fill
          sizes="16px"
          priority
          className="object-contain object-center block dark:hidden"
        />
      </div>
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
  docsRepositoryBase:
    "https://github.com/514-labs/moose/tree/main/apps/framework-docs",
  head: () => {
    const { asPath, defaultLocale, locale } = useRouter();
    const { frontMatter } = useConfig();
    const baseUrl =
      process.env.NEXT_PUBLIC_SITE_URL || "https://docs.fiveonefour.com";
    const url = `${baseUrl}${asPath !== "/" ? asPath : ""}`;

    // Determine which default OG image to use based on the path
    let defaultImage = "/og-image-fiveonefour.png"; // Default for root/main page
    if (asPath.startsWith("/moose")) {
      defaultImage = "/og-image-moose.png";
    } else if (asPath.startsWith("/aurora")) {
      defaultImage = "/og-image-aurora.png";
    }

    return (
      <>
        <title suppressHydrationWarning>
          {frontMatter.title || "514 Labs Documentation"}
        </title>
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta property="og:url" content={url} />
        <meta property="og:site_name" content="514 Labs Documentation" />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:site" content="@514hq" />
        <meta
          property="og:title"
          content={frontMatter.title || "514 Labs Documentation"}
        />
        <meta
          property="twitter:title"
          content={frontMatter.title || "514 Labs Documentation"}
        />
        <meta
          property="og:description"
          content={
            frontMatter.description ||
            "Documentation hub for Moose and Aurora, tools for building analytical backends and automated data engineering"
          }
        />
        <meta
          property="twitter:description"
          content={
            frontMatter.description ||
            "Documentation hub for Moose and Aurora, tools for building analytical backends and automated data engineering"
          }
        />
        <meta
          name="description"
          content={
            frontMatter.description ||
            "Documentation hub for Moose and Aurora, tools for building analytical backends and automated data engineering"
          }
        />
        {/* Use frontMatter.image if specified, otherwise use the default image based on path */}
        <meta
          property="og:image"
          content={`${baseUrl}${frontMatter.image || defaultImage}`}
        />
        <meta
          name="twitter:image"
          content={`${baseUrl}${frontMatter.image || defaultImage}`}
        />
        <link
          rel="icon"
          href="/favicon.ico"
          type="image/x-icon"
          sizes="16x16"
        />
        <link rel="canonical" href={url} />
        <Script
          src="https://buttons.github.io/buttons.js"
          strategy="afterInteractive"
        />
      </>
    );
  },
  navbar: {
    extraContent: () => (
      <div className="flex items-center gap-2 h-full" suppressHydrationWarning>
        <div className="max-h-7">
          <a
            className="github-button"
            href="https://github.com/514-labs/moose"
            data-color-scheme="no-preference: dark; light: light; dark: dark;"
            data-icon="octicon-star"
            data-size="large"
            data-show-count="true"
            aria-label="Star buttons/github-buttons on GitHub"
          >
            Star
          </a>
        </div>
      </div>
    ),
  },
  // main: ({ children }) => (
  //   <div className="relative">
  //     {children}
  //     <Contact />
  //   </div>
  // ),
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
    // Image component with zoom
    img: ({ src, alt, ...props }) => (
      <ImageZoom src={src} alt={alt || ""} {...props} />
    ),
    // Text components with direct styling
    p: ({ children, className, ...props }) => (
      <p className={cn("my-2", baseTextStyles.small, className)} {...props}>
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
      <li className={cn("list-item list-disc my-0 py-0", baseTextStyles.small)}>
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
      <Link
        href={href}
        className={cn(
          "text-moose-purple hover:text-moose-purple/90 transition-colors",
          className,
        )}
      >
        {children}
      </Link>
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
        <div className="flex flex-col gap-4 w-full">
          <p className={baseTextStyles.small}>
            MIT | {year} ©{" "}
            <Link
              href="https://fiveonefour.com"
              target="_blank"
              rel="noopener noreferrer"
              className="text-moose-purple hover:text-moose-purple/90 transition-colors"
            >
              Fiveonefour Labs Inc
            </Link>
          </p>
          <div className="flex flex-wrap items-center gap-4">
            <span className={baseTextStyles.small}>Follow us:</span>
            <div className="flex items-center gap-3">
              <Link
                href={PathConfig.github.path}
                target="_blank"
                rel="noopener noreferrer"
                className="text-moose-purple hover:text-moose-purple/90 transition-colors"
                aria-label="GitHub"
              >
                GitHub
              </Link>
              <Link
                href={PathConfig.twitter.path}
                target="_blank"
                rel="noopener noreferrer"
                className="text-moose-purple hover:text-moose-purple/90 transition-colors"
                aria-label="X (Twitter)"
              >
                Twitter
              </Link>
              <Link
                href={PathConfig.linkedin.path}
                target="_blank"
                rel="noopener noreferrer"
                className="text-moose-purple hover:text-moose-purple/90 transition-colors"
                aria-label="LinkedIn"
              >
                LinkedIn
              </Link>
              <Link
                href={PathConfig.youtube.path}
                target="_blank"
                rel="noopener noreferrer"
                className="text-moose-purple hover:text-moose-purple/90 transition-colors"
                aria-label="YouTube"
              >
                YouTube
              </Link>
              <Link
                href={PathConfig.slack.path}
                target="_blank"
                rel="noopener noreferrer"
                className="text-moose-purple hover:text-moose-purple/90 transition-colors"
                aria-label="Slack Community"
              >
                Slack
              </Link>
            </div>
          </div>
        </div>
      );
    },
  },
};
