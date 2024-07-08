"use client";

import { useQuery } from "@tanstack/react-query";
import Hierarchy from "@/components/ui/hierarchy";
import { fetchLogHierarchy } from "@/lib/data-fetch";

interface Props {
  selectedId: string | undefined;
  setSelectedId: React.Dispatch<React.SetStateAction<string | undefined>>;
}
export default function LogHierarchy({ selectedId, setSelectedId }: Props) {
  const { data: hierarchyData, isFetched } = useQuery({
    queryKey: ["hierarchy"],
    queryFn: () => fetchLogHierarchy(),
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
