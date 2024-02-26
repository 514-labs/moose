import { getCliData } from "./db";
import { unstable_noStore as noStore } from "next/cache";
import OverviewCard from "components/overview-card";
import { getModelFromRoute, getModelFromTable } from "lib/utils";

export default async function OverviewPage(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  return (
    <section className="p-4 grow overflow-y-scroll">
      <div className="text-8xl">Overview</div>
      <div className="">
        <div className="text-lg py-6">Primitives</div>
        <div className="grid grid-cols-3 gap-4">
          <div className="col-span-3 xl:col-span-1">
            <OverviewCard
              title="Models"
              numItems={data.models.length}
              link="/primitives/models"
              items={data.models.slice(0, 4).map((model) => ({
                name: model.name,
                link: `/primitives/models/${model.name}`,
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
              numItems={data.ingestionPoints.length}
              link="infrastructure/ingestion-points"
              items={data.ingestionPoints.slice(0, 4).map((ingestionPoint) => {
                return {
                  name: ingestionPoint.route_path,
                  link: `/primitives/models/${getModelFromRoute(ingestionPoint, data).name}?tab=usage`,
                };
              })}
            />
          </div>
          <div className="col-span-3 xl:col-span-1">
            <OverviewCard
              title="Tables"
              numItems={
                data.tables.filter(
                  (t) =>
                    t.engine !== "MaterializedView" && t.engine !== "Kafka",
                ).length
              }
              link="infrastructure/databases/tables?type=table"
              items={data.tables
                .filter(
                  (t) =>
                    t.engine !== "MaterializedView" && t.engine !== "Kafka",
                )
                .slice(0, 4)
                .map((table) => {
                  return {
                    name: table.name,
                    link: `/primitives/models/${getModelFromTable(table, data).name}?tab=query`,
                  };
                })}
            />
          </div>
          <div className="col-span-3 xl:col-span-1">
            <OverviewCard
              title="Views"
              numItems={
                data.tables.filter(
                  (t) =>
                    t.engine === "MaterializedView" &&
                    !t.name.includes("Kafka"),
                ).length
              }
              link="infrastructure/databases/tables?type=view"
              items={data.tables
                .filter((t) => t.engine === "MaterializedView")
                .slice(0, 4)
                .map((table) => {
                  return {
                    name: table.name,
                    link: `/primitives/models/${getModelFromTable(table, data).name}?tab=query`,
                  };
                })}
            />
          </div>
        </div>
      </div>
    </section>
  );
}
