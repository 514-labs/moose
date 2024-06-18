"use client";
import { TrackLink } from "@514labs/design-system-components/trackable-components";
import { usePathname } from "next/navigation";
import { Fragment } from "react";

export function NavBreadCrumb() {
  const pathName = usePathname();
  const { breadcrumbs } = pathName
    .split("/")
    .filter((string) => string != "")
    .reduce<{ breadcrumbs: { name: string; path: string }[]; path: string }>(
      (acc, curr) => ({
        breadcrumbs: [
          ...acc.breadcrumbs,
          { name: curr, path: `${acc.path}/${curr}` },
        ],
        path: `${acc.path}/${curr}`,
      }),
      { breadcrumbs: [], path: "" },
    );

  return (
    <div className="text-base text-muted-foreground flex">
      {breadcrumbs.map(({ name, path }, index) => (
        <Fragment key={index}>
          {index != 0 && <div className="px-1">/</div>}
          <TrackLink
            name="Link"
            subject={name}
            className={`capitalize ${path == pathName ? "text-foreground" : "hover:text-white"}`}
            href={path}
          >
            {" "}
            {name}
          </TrackLink>
        </Fragment>
      ))}
    </div>
  );
}
