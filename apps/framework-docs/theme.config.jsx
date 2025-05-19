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
import { LanguageSwitcher } from "./src/components/language-switcher";
import { ImageZoom } from "nextra/components";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { useRouter } from "next/router";
import { useConfig } from "nextra-theme-docs";

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
        src="/logo-light.png"
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
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta property="og:url" content={url} />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:site" content="@514hq" />
        <meta
          property="og:title"
          content={frontMatter.title || "514 Labs Documentation"}
        />
        <meta
          property="og:description"
          content={
            frontMatter.description ||
            "Documentation for Moose and Aurora by 514 Labs"
          }
        />
        <meta
          name="description"
          content={
            frontMatter.description ||
            "Documentation for Moose and Aurora by 514 Labs"
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
        <script async defer src="https://buttons.github.io/buttons.js"></script>
        <link rel="canonical" href={url} />
      </>
    );
  },
  navbar: {
    extraContent: () => (
      <div className="flex items-center gap-2 h-full">
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
        <Link href="https://www.boreal.cloud/sign-in">
          <Button variant="default">Sign In</Button>
        </Link>
      </div>
    ),
  },
  // main: ({ children }) => (
  //   <div className="relative">
  //     <div className="absolute right-0 top-0 z-10">
  //       <LanguageSwitcher />
  //     </div>
  //     {children}
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
