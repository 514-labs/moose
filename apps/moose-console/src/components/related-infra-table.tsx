import { Fragment } from "react";
import { Separator } from "./ui/separator";
import { DataModel, MooseObject } from "app/types";
import { TrackLink } from "@514labs/design-system/trackable-components";

export default function RelatedInfraTable({
  model,
  mooseObject,
}: {
  model: DataModel;
  mooseObject: MooseObject;
}) {
  const { table, ingestion_point, model: relatedModel } = model;
  return (
    <Fragment>
      {mooseObject != MooseObject.Table && table && (
        <TrackLink
          name="Link"
          subject="table"
          href={`/infrastructure/databases/${table.database}/tables/${table.uuid}`}
        >
          <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
            <div className="flex flex-row grow">
              <div className="flex grow py-4 space-x-4">
                <div className="grow basis-1">{table.name}</div>
                <div className="grow basis-1 text-muted-foreground">
                  {table.name.includes("view") ? "view" : "table"}
                </div>
              </div>
            </div>
            <Separator />
          </div>
        </TrackLink>
      )}
      {mooseObject != MooseObject.Model && table && (
        <TrackLink
          name="Link"
          subject="model"
          href={`/primitives/models/${relatedModel.name}`}
        >
          <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
            <div className="flex flex-row grow">
              <div className="flex grow py-4 space-x-4">
                <div className="grow basis-1">{table.name}</div>
                <div className="grow basis-1 text-muted-foreground">
                  {table.name.includes("view") ? "view" : "table"}
                </div>
              </div>
            </div>
            <Separator />
          </div>
        </TrackLink>
      )}
      {mooseObject != MooseObject.IngestionPoint && (
        <TrackLink
          name="Link"
          subject="ingestion-point"
          href={`/infrastructure/ingestion-points/${ingestion_point.route_path.split("/").at(-2)}`}
        >
          <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
            <div className="flex flex-row grow">
              <div className="flex grow py-4 space-x-4">
                <div className="grow basis-1">{ingestion_point.route_path}</div>
                <div className="grow basis-1 text-muted-foreground">
                  ingestion point
                </div>
              </div>
            </div>
          </div>
        </TrackLink>
      )}
    </Fragment>
  );
}
