"use client";
import FilterBar from "@/components/filter-bar";
import LogHierarchy from "@/components/log-hierarchy";
import LogTable from "@/components/log-table";
import { useState } from "react";

export default function Home() {
  const [selectedSource, setSelectedSource] = useState<string | undefined>();
  const [search, setSearch] = useState<string>("");
  return (
    <main className="flex min-h-screen h-screen flex-col items-center justify-between grid grid-cols-5">
      <div className="col-span-1 h-full overflow-scroll">
        <LogHierarchy
          selectedId={selectedSource}
          setSelectedId={setSelectedSource}
        />
      </div>
      <div className="col-span-4 w-full">
        <FilterBar setSearch={setSearch} />
        <LogTable search={search} source={selectedSource} />
      </div>
    </main>
  );
}
