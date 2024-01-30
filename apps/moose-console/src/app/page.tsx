
import { Separator } from "components/ui/separator";
import { Button } from "components/ui/button";
import Link from "next/link";
import { getCliData } from "./db";
import { unstable_noStore as noStore } from 'next/cache';




export default async function Primitives(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  return (
    <section className="p-4">
      <div className="text-5xl py-10">Overview</div>
      <div>
        <div className="text-3xl py-6 text-muted-foreground">Primitives | Docs</div>
        <div className="flex flex-row space-x-5">
          <div className="flex-1">
            <div className="text-4xl py-4">{data.models.length} Models</div>
            <Separator />
            {data.models.slice(0,10).map((model, index) => (
              <div key={index}>
                <div className="py-4 text-muted-foreground">{model.name}</div>
                <Separator />
              </div>
            ))}
            <div className="py-5">
              <Link href="/primitives/models">
                <Button className="border-primary" variant="outline">More</Button>
              </Link>
            </div>
          </div>
          <div className="flex-1">
            <div className="text-4xl py-4">Flows</div>
            <Separator />

          </div>
          <div className="flex-1">
            <div className="text-4xl py-4">Insights</div>
            <Separator />

          </div>
        </div>
      </div>
      
      <div>
        <div className="text-3xl py-6 text-muted-foreground">Infrastructure | Docs</div>
        <div className="flex flex-row space-x-5">
          <div className="flex-1">
            <div className="text-4xl py-4">{data.ingestionPoints.length} Ingestion Points</div>
            <Separator />
            {data.ingestionPoints.slice(0,10).map((ingestionPoint, index) => (
              <div key={index}>
                <div className="py-4 text-muted-foreground">{ingestionPoint.route_path}</div>
                <Separator />
              </div>
            ))}
            <div className="py-5">
              <Link href="/infrastructure/ingestion-points">
                <Button className="border-primary" variant="outline">More</Button>
              </Link>
            </div>
          </div>
          <div className="flex-1">
            <div className="text-4xl py-4">{data.queues.length} Queues</div>
            <Separator />
            {data.queues.slice(0,10).map((queue, index) => (
              <div key={index}>
                <div className="py-4 text-muted-foreground">{queue}</div>
                <Separator />
              </div>
            ))}
            <div className="py-5">
              <Button className="border-primary"  variant="outline">More</Button>
            </div>

          </div>
          <div className="flex-1">
            <div className="text-4xl py-4">{
              // Get the number of databases from the uniques in the tables
              new Set(data.tables.map((table) => table.database)).size
              } Tables & Views </div>
            <Separator />
            {data.tables.slice(0,10).map((table, index) => (
              <Link href={`/infrastructure/databases/${table.database}/tables/${table.uuid}`} key={index}>
                <div >
                  <div className="py-4 text-muted-foreground">{table.name}</div>
                  <Separator />
                </div>
              </Link>
            ))}
            <div className="py-5">
              <Link href="/infrastructure/databases">
                <Button className="border-primary" variant="outline">More</Button>
              </Link>
            </div>

          </div>
        </div>
      </div>
    </section>    
  );
}
