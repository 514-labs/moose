"use client";

import { useQuery } from "@tanstack/react-query";
import Hierarchy from "@/components/ui/hierarchy";
import { fetchLogHierarchy } from "@/lib/data-fetch";
import { SeverityLevel } from "@/lib/utils";

interface Props {
  selectedId: string | undefined;
  source: string | undefined;
  search: string;
  severity: SeverityLevel[];
  setSelectedId: React.Dispatch<React.SetStateAction<string | undefined>>;
}
export default function LogHierarchy({
  selectedId,
  setSelectedId,
  search,
  severity,
}: Props) {
  const { data: hierarchyData, isFetched } = useQuery({
    queryKey: ["hierarchy", search, severity],
    queryFn: () => fetchLogHierarchy({ search, severity }),
    initialData: [],
  });

  return (
    <Hierarchy
      selectedId={selectedId}
      setSelectedId={setSelectedId}
      toc={hierarchyData}
    />
  );
}
