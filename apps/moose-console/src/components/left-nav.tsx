"use client"
// Create a left nav from the navigation menu primitives in /ui/menu-bar.tsx

import { usePathname } from 'next/navigation'
import { cn } from "lib/utils";
import Link from "next/link";

import {
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
  } from "components/ui/accordion"

interface NavLinks {
    name: string;
    href: string;
    subLinks?: NavLinks[];
}

interface Section {
    name: string;
    href: string;
    links?: NavLinks[];
}

interface LeftNavProps {
    sections: Section[];
}


// There are two sections, a "Primitives" section and a "Infrastructure" Section. 
// 
// The "Primitives" section has the following links:
// models, flows and insights

// The "Infrastructure" section has the following links:
// Ingestion points
// Queues
// Tables
// Views

const sections: Section[] = [
    {
        name: "Overview",
        href: "/",
    },
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
        name: "Infrastructure",
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

export const ExpandableItem = ({ name, href, subLinks }: NavLinks) => {
    const pathname = usePathname()

    return (
        <Accordion key={name} type="single" defaultValue="item-1" collapsible>
            <AccordionItem value="item-1" className="border-b-0">
                <AccordionTrigger className={cn('py-0 font-normal flex-grow-0 space-x-4', pathname == href ? "text-forefround" : "text-muted-foreground")}>
                    <li  className="py-2">
                        <Link href={href} className=" hover:text-foreground block">{name}</Link>
                    </li>
                </AccordionTrigger>
                <AccordionContent className="pl-2 font-normal">
                    <ul>
                        {subLinks?.map((link) => (
                            Item(link)
                        ))}
                    </ul>
                </AccordionContent>
            </AccordionItem>
        </Accordion>
    )
}

export const Item = ({ name, href }: NavLinks) => {
    const pathname = usePathname()

    return (
        <li key={name} className={cn("py-2 text-muted-foreground", pathname == href ? "text-forefround" : "")}>
            <Link href={href} className=" hover:text-foreground block">{name}</Link>
        </li>
    )
}


export const LeftNav = () => {
    const pathname = usePathname()

    return (
        <div className="flex flex-col w-64 py-8">
            {sections.map((section) => (
                <div key={section.name} className="mb-10">
                    <Link href={section.href} className="text-lg">{section.name}</Link>
                    <ul className="mt-3">
                        {section.links && section.links.map((link) => (
                            link.subLinks ? ExpandableItem(link) : Item(link)
                        ))} 
                    </ul>
                </div>)
            )}
        </div>
    )
}