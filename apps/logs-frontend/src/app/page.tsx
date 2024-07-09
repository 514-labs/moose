"use client";
import FilterBar from "@/components/filter-bar";
import LogHierarchy from "@/components/log-hierarchy";
import LogTable from "@/components/log-table";
import { SeverityLevel } from "@/lib/utils";
import { useState } from "react";
import { useSession, signIn, signOut } from "next-auth/react";

export default function Home() {
  const [selectedSource, setSelectedSource] = useState<string | undefined>();
  const [search, setSearch] = useState<string>("");
  const [severity, setSeverity] = useState<SeverityLevel[]>([
    SeverityLevel.ERROR,
    SeverityLevel.WARN,
  ]);
  const { data: session } = useSession();
  if (!session) {
    return (
      <>
        <p>Not signed in</p>
        <br />
        <button onClick={() => signIn()}>Sign in</button>
      </>
    );
  }
  return (
    <main className="flex min-h-screen h-screen w-screen flex-col items-center justify-between grid grid-cols-6">
      <div className="col-span-2 h-full overflow-scroll">
        <LogHierarchy
          severity={severity}
          search={search}
          source={selectedSource}
          selectedId={selectedSource}
          setSelectedId={setSelectedSource}
        />
      </div>
      <div className="col-span-4 w-full h-screen">
        <FilterBar
          setSeverity={setSeverity}
          severity={severity}
          setSearch={setSearch}
        />
        <LogTable severity={severity} search={search} source={selectedSource} />
      </div>
    </main>
  );
}
