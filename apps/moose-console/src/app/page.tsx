import { PrimitiveCard } from "components/primitive-card";
import { homeMock } from "./mock";
import { DatabasesCard } from "components/databases-overview-card";
import { QueuesCard } from "components/queues-overview-card";
import { IngestionPointsCard } from "components/ingestion-points-overview-card";
import { Separator } from "components/ui/separator";
import { Button, buttonVariants } from "components/ui/button";
import Link from "next/link";




export default async function Primitives(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  // noStore();
  // const data = await getCliData();
  const data = homeMock;

  return (
    <section className="p-4">
      <div className="text-5xl py-10">Overview</div>
      <div>
        <div className="text-3xl py-6 text-muted-foreground">Primitives | Docs</div>
        <div className="flex flex-row space-x-5">
          <div className="flex-1">
            <div className="text-4xl py-4">{data.modelMock.models.length} Models</div>
            <Separator />
            {data.modelMock.models.slice(0,4).map((model) => (
              <div >
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
            <div className="text-4xl py-4">{data.infrastructure.ingestionPoints.length} Ingestion Points</div>
            <Separator />
            {data.infrastructure.ingestionPoints.slice(0,4).map((ingestionPoint) => (
              <div >
                <div className="py-4 text-muted-foreground">{ingestionPoint.name}</div>
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
            <div className="text-4xl py-4">{data.infrastructure.queues.length} Queues</div>
            <Separator />
            {data.infrastructure.queues.slice(0,4).map((queue) => (
              <div >
                <div className="py-4 text-muted-foreground">{queue.name}</div>
                <Separator />
              </div>
            ))}
            <div className="py-5">
              <Button className="border-primary"  variant="outline">More</Button>
            </div>

          </div>
          <div className="flex-1">
            <div className="text-4xl py-4">{data.infrastructure.databases.length} Databases</div>
            <Separator />
            {data.infrastructure.databases.slice(0,4).map((database) => (
              <Link href={`/infrastructure/databases/${database.id}`}>
                <div >
                  <div className="py-4 text-muted-foreground">{database.name}</div>
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