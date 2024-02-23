"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Fragment } from "react";

export function NavBreadCrumb() {
    const pathName = usePathname();
    const { breadcrumbs } = pathName
        .split('/')
        .filter(string => string != '')
        .reduce((acc, curr) => ({ breadcrumbs: [...acc.breadcrumbs, { name: curr, path: `${acc.path}/${curr}` }], path: `${acc.path}/${curr}` }), { breadcrumbs: [], path: '' })

    return <div className="text-base text-muted-foreground flex">
        {breadcrumbs.map(({ name, path }, index) => <Fragment key={index}>
            {index != 0 && <div className="px-1">/</div>}
            <Link className={`capitalize ${path == pathName ? 'text-foreground' : 'hover:text-white'}`} href={path}> {name}</Link >
        </Fragment>)}
    </div>
}