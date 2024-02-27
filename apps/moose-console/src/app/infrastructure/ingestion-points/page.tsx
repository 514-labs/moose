import { getCliData } from "app/db";
import { IngestionPointsList } from "components/ingestion-points-list";
import { NavBreadCrumb } from "components/nav-breadcrumb";
import { unstable_noStore as noStore } from "next/cache";

export default async function IngestionPointsPage(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">
          {data.ingestionPoints.length} Ingestion Points
        </div>
      </div>
      <IngestionPointsList ingestionPoints={data.ingestionPoints} />
    </section>
  );
}
