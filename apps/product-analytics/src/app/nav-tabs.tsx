"use client";

import { usePathname } from "next/navigation";

export const NavTabs = () => {
  const pathName = usePathname();

  return (
    <div className="m-4">
      <a
        className={`p-3 m-2 hover:bg-accent ${pathName == "/" ? "border-b-2 border-foreground" : ""}`}
        href="/"
      >
        Dashboard
      </a>
      <a
        className={`p-3 m-2 hover:bg-accent ${pathName == "/insights" ? "border-b-2 border-foreground" : ""}`}
        href="/insights"
      >
        Reports
      </a>
    </div>
  );
};
