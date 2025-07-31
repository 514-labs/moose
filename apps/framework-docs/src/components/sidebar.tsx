"use client";

import { usePathname } from "next/navigation";
import type { PageMapItem } from "nextra";
import { normalizePages } from "nextra/normalize-pages";
import type { FC } from "react";
import Link from "next/link";

export const Sidebar: FC<{ pageMap: PageMapItem[] }> = ({ pageMap }) => {
  const pathname = usePathname();
  const { docsDirectories } = normalizePages({
    list: pageMap,
    route: pathname,
  });

  return (
    <div
      style={{
        background: "lightgreen",
        padding: 20,
      }}
    >
      <h3>Sidebar</h3>
      <ul>
        {docsDirectories.map(function renderItem(item) {
          const route =
            item.route || ("href" in item ? (item.href as string) : "");
          const { title } = item;
          return (
            <li>
              {"children" in item ?
                <details>
                  <summary>{title}</summary>
                  {item.children.map((child) => renderItem(child))}
                </details>
              : <Link href={route} style={{ textDecoration: "none" }}>
                  {title}
                </Link>
              }
            </li>
          );
        })}
      </ul>
    </div>
  );
};
