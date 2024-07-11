"use client";

import {
  SeverityLevel,
  cn,
  severityLevelColors,
  severityLevels,
} from "@/lib/utils";
import { Input } from "./ui/input";
import { ToggleGroup, ToggleGroupItem } from "./ui/toggle-group";
import { Dispatch, SetStateAction } from "react";
import { Combobox } from "./ui/creatable-select";

interface Props {
  setSearch: (search: string) => void;
  setSeverity: Dispatch<SetStateAction<SeverityLevel[]>>;
  setMachineId: (id: string) => void;
  severity: SeverityLevel[];
}
export default function FilterBar({ setSearch, setSeverity, severity }: Props) {
  return (
    <div className="flex flex-row items-center justify-between bg-gray-100 p-2 gap-2">
      <Input
        placeholder="Search..."
        onChange={(e) => setSearch(e.target.value)}
      />
      <Combobox
        options={[
          { val: "Select", label: "blah" },
          { val: "othero", label: "blahtwo" },
        ]}
        onOpenChange={(value) => console.log("open change")}
        placeholder="Select Machine"
        selected={""} // string or array
        onSelect={(select) => console.log("select", select)}
        onCreate={(create) => console.log(create, "create")}
      />
      <ToggleGroup
        variant="outline"
        type="multiple"
        value={severity}
        onValueChange={(v) => setSeverity(v as SeverityLevel[])}
      >
        {severityLevels.map((level) => (
          <ToggleGroupItem
            defaultChecked={severity.find((sev) => sev == level) ? true : false}
            className="p-0 m-0 w-16 h-10"
            key={level}
            value={level}
          >
            <div
              className={`flex justify-center rounded-lg items-center w-full h-full ${
                severity.find((sev) => sev == level)
                  ? severityLevelColors[level]
                  : ""
              }`}
            >
              {level}
            </div>
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}
