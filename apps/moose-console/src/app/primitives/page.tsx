import { getCliData } from "app/db";
import { unstable_noStore as noStore } from "next/cache";
import OverviewCard from "components/overview-card";

export default async function PrimitivesPage(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  return (
    <section className="p-4 grow">
      <div className="text-8xl py-10">Primitives</div>
      <div className="mb-20">
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
    </section>
  );
}
