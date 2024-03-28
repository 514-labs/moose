"use client";

import { useContext } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { CURRENT_VERSION } from "app/types";
import { VersionContext } from "version-context";
import { usePathname } from "next/navigation";

function shouldSwitcherBeDisabled(pathname: string) {
  const regex = /^\/primitives\/models/;

  if (regex.test(pathname)) {
    return false;
  }

  switch (pathname) {
    case "/":
    case "/primitives":
    case "/infrastructure":
      return false;
    default:
      return true;
  }
}

export default function VersionSelect() {
  const pathName = usePathname();
  const { version, setVersion, cliData } = useContext(VersionContext);

  const versions = Object.keys(cliData?.past || []);

  return (
    <div className="content-center m-4 w-24">
      <Select
        disabled={shouldSwitcherBeDisabled(pathName)}
        onValueChange={(val) => setVersion(val)}
        value={version}
      >
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={CURRENT_VERSION}>latest</SelectItem>
          {versions.map((version) => (
            <SelectItem key={version} value={version}>
              v{version}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
