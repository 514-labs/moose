"use client";

import * as React from "react";

import { cn } from "lib/utils";
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  navigationMenuTriggerStyle,
} from "components/ui/navigation-menu";
import { usePathname } from "next/navigation";

interface Section {
  name: string;
  href: string;
  links?: NavLinks[];
}

interface NavLinks {
  name: string;
  href: string;
  subLinks?: NavLinks[];
}

const sections: Section[] = [
  {
    name: "Primitives",
    href: "/primitives",
    links: [
      {
        name: "Models",
        href: "/primitives/models",
      },
      {
        name: "Flows",
        href: "/primitives/flows",
      },
      {
        name: "Insights",
        href: "/primitives/insights",
      },
    ],
  },
  {
    name: "Infra",
    href: "/infrastructure",
    links: [
      {
        name: "Ingestion Points",
        href: "/infrastructure/ingestion-points",
      },
      {
        name: "Tables",
        href: "/infrastructure/databases/tables?type=table",
      },
      {
        name: "Views",
        href: "/infrastructure/databases/tables?type=view",
      },
    ],
  },
];

const NavItem = (section: Section, path: string, key: number) => {
  return (
    <NavigationMenuItem key={key}>
      <NavigationMenuTrigger
        className={cn(
          "text-lg font-normal rounded-none",
          section.href.split("/").at(1) === path.split("/").at(1)
            ? "text-foreground border-b-2 border-b-foreground"
            : "text-muted-foreground",
        )}
      >
        <NavigationMenuLink href={section.href}>
          {section.name}
        </NavigationMenuLink>
      </NavigationMenuTrigger>
      <NavigationMenuContent>
        {section.links?.map((link, index) => (
          <div className="py-6 px-4 w-[500px]" key={index}>
            <NavigationMenuLink href={link.href}>
              {link.name}
            </NavigationMenuLink>
          </div>
        ))}
      </NavigationMenuContent>
    </NavigationMenuItem>
  );
};

export const TopNavMenu = () => {
  const path = usePathname();
  return (
    <NavigationMenu>
      <NavigationMenuList>
        <NavigationMenuItem
          className={cn(
            navigationMenuTriggerStyle(),
            "text-lg font-normal rounded-none",
            path === "/"
              ? "text-foreground border-b-2 border-b-foreground"
              : "text-muted-foreground",
          )}
        >
          <NavigationMenuLink href="/">Overview</NavigationMenuLink>
        </NavigationMenuItem>
        {sections.map((section, index) => NavItem(section, path, index))}
        <NavigationMenuItem>
          <NavigationMenuLink
            href="https://docs.moosejs.com"
            className={cn(
              navigationMenuTriggerStyle(),
              "text-lg text-muted-foreground font-normal rounded-none",
            )}
          >
            Docs
          </NavigationMenuLink>
        </NavigationMenuItem>
      </NavigationMenuList>
    </NavigationMenu>
  );
};
