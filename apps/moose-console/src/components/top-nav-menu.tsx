"use client";

import * as React from "react";

import { cn } from "lib/utils";
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuList,
  NavigationMenuTrigger,
  navigationMenuTriggerStyle,
} from "components/ui/navigation-menu";
import { usePathname } from "next/navigation";
import { NavigationMenuLinkTrack } from "./trackable-components";

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
        <NavigationMenuLinkTrack
          name="Nav Link"
          subject={section.name}
          href={section.href}
        >
          {section.name}
        </NavigationMenuLinkTrack>
      </NavigationMenuTrigger>
      <NavigationMenuContent>
        {section.links?.map((link, index) => (
          <div className="py-6 px-4 w-[500px]" key={index}>
            <NavigationMenuLinkTrack
              name="Nav Link"
              subject={link.name}
              href={link.href}
            >
              {link.name}
            </NavigationMenuLinkTrack>
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
          <NavigationMenuLinkTrack name="Nav Link" subject="Overview" href="/">
            Overview
          </NavigationMenuLinkTrack>
        </NavigationMenuItem>
        {sections.map((section, index) => NavItem(section, path, index))}
        <NavigationMenuItem>
          <NavigationMenuLinkTrack
            name="Nav Link"
            subject="Docs"
            href="https://docs.moosejs.com"
            className={cn(
              navigationMenuTriggerStyle(),
              "text-lg text-muted-foreground font-normal rounded-none",
            )}
          >
            Docs
          </NavigationMenuLinkTrack>
        </NavigationMenuItem>
      </NavigationMenuList>
    </NavigationMenu>
  );
};
