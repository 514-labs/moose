import { DataTable } from "components/data-table";
import React from "react";
import { viewColumns } from "./columns";
import { infrastructureMock } from "../mock";

const Placeholder: React.FC = () => {
  const data = infrastructureMock;

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
      <div className="text-9xl py-20">Views</div>
      <div className="flex-row space-x-3">
        <DataTable
          columns={viewColumns}
          data={data.databases.flatMap((x) =>
            x.views.map((t) => ({ databaseId: x.id, ...t })),
          )}
        />
      </div>
    </section>
  );
};

export default Placeholder;
