"use client"

import * as React from "react"
import Link from "next/link"

import { cn } from "lib/utils"
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  NavigationMenuViewport,
  navigationMenuTriggerStyle,
} from "components/ui/navigation-menu"

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
      ]
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
              name: "Queues",
              href: "/infrastructure/queues",
          },
          {
              name: "Databases",
              href: "/infrastructure/databases",
              subLinks: [
                  {
                      name: "Tables",
                      href: "/infrastructure/tables",
                  },
                  {
                      name: "Views",
                      href: "/infrastructure/views",
                  }
              ]
          }
          
      ]
  }
]

const NavItem = (section: Section) => {
  return (
    <NavigationMenuItem>
      <NavigationMenuTrigger>
        <NavigationMenuLink href={section.href} className="text-lg font-normal">{section.name}</NavigationMenuLink>
      </NavigationMenuTrigger>
      <NavigationMenuContent>
        {section.links?.map((link, index) => (
          <div className="py-6 px-4 w-[500px]" key={index}>
            <NavigationMenuLink href={link.href} >{link.name}</NavigationMenuLink>
          </div>
        ))}
      </NavigationMenuContent>
    </NavigationMenuItem>
  )
}


export const TopNavMenu = () => {
  return (
    <NavigationMenu>
      <NavigationMenuList >
        <NavigationMenuItem >
          <NavigationMenuLink href="/" className={cn(navigationMenuTriggerStyle(), "text-lg font-normal")}>
                Overview
          </NavigationMenuLink>
        </NavigationMenuItem>
        {sections.map((section, index) => (
          <NavItem {...section} key={index}/>
        ))}
        <NavigationMenuItem>
          <NavigationMenuLink href="/docs" className={cn(navigationMenuTriggerStyle(), "text-lg font-normal")}>
                Docs
          </NavigationMenuLink>
        </NavigationMenuItem>
      </NavigationMenuList>
      
    </NavigationMenu>
  )
}




