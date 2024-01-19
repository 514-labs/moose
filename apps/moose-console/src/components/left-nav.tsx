// Create a left nav from the navigation menu primitives in /ui/menu-bar.tsx

interface NavLinks {
    name: string;
    href: string;
}

interface Section {
    name: string;
    href: string;
    links: NavLinks[];
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

let sections: Section[] = [
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
                name: "Tables",
                href: "/infrastructure/tables",
            },
            {
                name: "Views",
                href: "/infrastructure/views",
            },
        ]
    }
]


export const LeftNav = () => {
    return (
        <div className="flex flex-col w-64 py-8">
            {sections.map((section) => (
                <div key={section.name} className="mb-10">
                    <div className="text-lg">{section.name}</div>
                    <ul className="mt-3">
                        {section.links.map((link) => (
                            <li key={link.name} className="py-2 text-muted-foreground">
                                <a href={link.href} className=" hover:text-foreground block">{link.name}</a>
                            </li>
                        ))} 
                    </ul>
                </div>
            )
            )}
        </div>
    )
}