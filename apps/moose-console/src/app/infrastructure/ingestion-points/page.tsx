import { getCliData } from "app/db";
import { IngestionPointsList } from "components/ingestion-points-list";
import { unstable_noStore as noStore } from 'next/cache';
import Link from "next/link";

export default async function IngestionPointsPage(): Promise<JSX.Element> {
    // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();
  
    return (
      <section className="p-4 max-h-screen overflow-y-auto grow">
        <div className="py-10">
          <div className="text-6xl">
              <Link className="text-muted-foreground" href={"/"}>overview/</Link>
              Ingestion Points
          </div>
        </div>
        <IngestionPointsList ingestionPoints={data.ingestionPoints}/>
      </section>    
    );
  }