"use client";

import {
  ParsedLogsResponse,
  fetchLogs,
  createLogColumns,
} from "@/lib/data-fetch";
import { useInfiniteQuery, keepPreviousData } from "@tanstack/react-query";
import { SortingState } from "@tanstack/react-table";
import { useMemo, useState } from "react";
import { InfiniteTable } from "./ui/infinite-table";
import { SeverityLevel } from "@/lib/utils";

interface Props {
  source: string | undefined;
  search: string;
  severity: SeverityLevel[];
}
export default function LogTable({ source, search, severity }: Props) {
  const fetchSize = 40;
  const [sorting, setSorting] = useState<SortingState>([
    { id: "date", desc: true },
  ]);
  const { data, fetchNextPage, isFetching, isLoading } =
    useInfiniteQuery<ParsedLogsResponse>({
      queryKey: [
        "",
        sorting, //refetch when sorting changes
        source, //refetch when source changes
        search,
        severity,
      ],
      queryFn: async ({ pageParam = 0 }) => {
        const start = (pageParam as number) * fetchSize;
        const fetchedData = await fetchLogs({
          limit: fetchSize,
          offset: start,
          sorting,
          source,
          search,
          severity,
        });
        return fetchedData;
      },
      initialPageParam: 0,
      getNextPageParam: (_lastGroup, groups) => groups.length,
      refetchOnWindowFocus: false,
      placeholderData: keepPreviousData,
    });

  //flatten the array of arrays from the useInfiniteQuery hook
  const flatData = useMemo(
    () => data?.pages?.flatMap((page) => page.data) ?? [],
    [data],
  );
  const totalDBRowCount = data?.pages?.[0]?.meta?.totalRowCount ?? 0;
  const totalFetched = flatData.length;

  if (!data) return null;

  const logColumns = createLogColumns({ selectedSource: source });

  return (
    <InfiniteTable
      columns={logColumns}
      data={flatData}
      isFetching={isFetching}
      isLoading={isLoading}
      fetchNextPage={fetchNextPage}
      totalDBRowCount={totalDBRowCount}
      totalFetched={totalFetched}
      sorting={sorting}
      setSorting={setSorting}
    />
  );
}
