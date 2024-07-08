"use client";

import { Input } from "./ui/input";

interface Props {
  setSearch: (search: string) => void;
}
export default function FilterBar({ setSearch }: Props) {
  return (
    <div className="flex flex-row items-center justify-between bg-gray-100 p-2">
      <Input onChange={(e) => setSearch(e.target.value)} />
    </div>
  );
}
