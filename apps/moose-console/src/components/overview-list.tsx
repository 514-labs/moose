import OverviewCard from "components/overview-card";
import { DataModel } from "app/types";

export function PrimitivesOverviewList({ models }: { models: DataModel[] }) {
  const flows = models
    .filter((model) => model.flows && model.flows.length > 0)
    .flatMap((model) =>
      model.flows.map((flow) => ({
        name: `${model.model.name} to ${flow}`,
        link: "",
      })),
    );

  return (
    <div className="">
      <div className="text-lg py-6">Primitives</div>
      <div className="grid grid-cols-3 gap-4">
        <div className="col-span-3 xl:col-span-1">
          <OverviewCard
            title="Models"
            numItems={models.length}
            link="/primitives/models"
            items={models.slice(0, 4).map((model) => ({
              name: model.model.name,
              link: `/primitives/models/${model.model.name}`,
            }))}
          />
        </div>
        <div className="col-span-3 xl:col-span-1 flex flex-col">
          <OverviewCard
            title="Flows"
            numItems={flows.length}
            link="/primitives/flows"
            items={flows}
          />
        </div>
        <div className="col-span-3 xl:col-span-1 flex flex-col">
          <OverviewCard
            title="Insights"
            numItems={0}
            link="/primitives/insights"
            items={[]}
          />
        </div>
      </div>
    </div>
  );
}

export function InfrastructureOverviewList({
  models,
}: {
  models: DataModel[];
}) {
  const views = models
    .filter(({ table }) => table.engine == "View")
    .slice(0, 4);

  const tables = models
    .filter(({ table }) => table.engine != "View")
    .slice(0, 4);

  return (
    <div>
      <div className="text-lg py-6">Infrastructure</div>
      <div className="grid grid-cols-3 gap-4">
        <div className="col-span-3 xl:col-span-1">
          <OverviewCard
            title="Ingestion Points"
            numItems={models.length}
            link="infrastructure/ingestion-points"
            items={models.slice(0, 4).map(({ ingestion_point }) => ({
              name: ingestion_point.route_path,
              link: `/infrastructure/ingestion-points/${ingestion_point.route_path.split("/").at(-2)}`,
            }))}
          />
        </div>
        <div className="col-span-3 xl:col-span-1">
          <OverviewCard
            title="Tables"
            numItems={tables.length}
            link="infrastructure/databases/tables?type=table"
            items={tables.map(({ table }) => ({
              name: table.name,
              link: `/infrastructure/databases/${table.database}/tables/${table.uuid}`,
            }))}
          />
        </div>
        <div className="col-span-3 xl:col-span-1">
          <OverviewCard
            title="Views"
            numItems={views.length}
            link="infrastructure/databases/tables?type=view"
            items={views.map(({ table }) => {
              return {
                name: table.name,
                link: `/infrastructure/databases/${table.database}/tables/${table.uuid}`,
              };
            })}
          />
        </div>
      </div>
    </div>
  );
}
