import { Fragment } from "react";
import Link from "next/link";
import { Separator } from "./ui/separator";
import { Infra } from "app/db";

export default function RelatedInfraTable({ infra }: { infra: Infra }) {
  return (
    <Fragment>
      {infra &&
        infra.tables.map((table, index) => (
          <Link
            key={index}
            href={`/infrastructure/databases/${table.database}/tables/${table.uuid}`}
          >
            <div
              key={index}
              className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer"
            >
              <div className="flex flex-row grow">
                <div key={index} className="flex grow py-4 space-x-4">
                  <div className="grow basis-1">{table.name}</div>
                  <div className="grow basis-1 text-muted-foreground">
                    {table.name.includes("view") ? "view" : "table"}
                  </div>
                </div>
              </div>
              <Separator />
            </div>
          </Link>
        ))}
      {infra &&
        infra.ingestionPoints.map((ingestionPoint, index) => (
          <Link
            key={index}
            href={`/infrastructure/ingestion-points/${ingestionPoint.route_path.split("/").at(-1)}`}
          >
            <div
              key={index}
              className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer"
            >
              <div className="flex flex-row grow">
                <div key={index} className="flex grow py-4 space-x-4">
                  <div className="grow basis-1">
                    {ingestionPoint.route_path}
                  </div>
                  <div className="grow basis-1 text-muted-foreground">
                    ingestion point
                  </div>
                </div>
              </div>
              {index === infra.ingestionPoints.length && <Separator />}
            </div>
          </Link>
        ))}
    </Fragment>
  );
}
