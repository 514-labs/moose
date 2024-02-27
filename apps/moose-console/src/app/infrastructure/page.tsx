import { getCliData } from "app/db";
import OverviewCard from "components/overview-card";
import { Metadata } from "next";
import { unstable_noStore as noStore } from "next/cache";

export const metadata: Metadata = {
  title: "MooseJS | Build for the modern data stack",
  openGraph: {
    images: "/open-graph/og_igloo_4x.webp",
  },
};

export default async function Infrastructure(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();
  return (
    <section className="p-4 grow overflow-y-scroll">
      <div>
        <div className="text-8xl py-6">Infrastructure</div>
        <div className="grid grid-cols-3 gap-4">
          <div className="col-span-3 xl:col-span-1">
            <OverviewCard
              title="Ingestion Points"
              numItems={data.ingestionPoints.length}
              link="infrastructure/ingestion-points"
              items={data.ingestionPoints.slice(0, 4).map((ingestionPoint) => {
                return {
                  name: ingestionPoint.route_path,
                  link: `/infrastructure/ingestion-points/${ingestionPoint.route_path.split("/").at(-1)}`,
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
                    link: `/infrastructure/databases/${table.database}/tables/${table.uuid}`,
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
