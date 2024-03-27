import { getCliData } from "./db";
import { unstable_noStore as noStore } from "next/cache";
import OverviewCard from "components/overview-card";

export default async function OverviewPage(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  const models = data.current.models;
  const views = models
    .filter(({ table }) => table.engine == "View")
    .slice(0, 4);

  const tables = models
    .filter(({ table }) => table.engine != "View")
    .slice(0, 4);

  return (
    <section className="p-4 grow overflow-y-scroll">
      <div className="text-8xl">Overview</div>
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
              numItems={0}
              link="/primitives/flows"
              items={[]}
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
                link: `/infrastructure/ingestion-points/${ingestion_point.route_path.split("/").at(-1)}`,
              }))}
            />
          </div>
          <div className="col-span-3 xl:col-span-1">
            <OverviewCard
              title="Tables"
              numItems={models.length}
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
              numItems={models.length}
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
    </section>
  );
}
