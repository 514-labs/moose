import { PrimitiveCard } from "components/primitive-card";
import { homeMock } from "./mock";
import { DatabasesCard } from "components/databases-card";
import { QueuesCard } from "components/queues-card";
import { IngestionPointsCard } from "components/ingestion-points";
  


export default async function Primitives(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  // noStore();
  // const data = await getCliData();
  const data = homeMock;

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
      <div className="text-9xl py-20">Overview</div>
      <div className="text-6xl py-10">Primitives</div>
      <div className="flex flex-row space-x-3">
        {data.primitives.map((primitive, index) => (
          <PrimitiveCard key={index} primitive={primitive}/>
        ))}
      </div>
      <div className="text-6xl py-10">Infrastructure</div>
      <div className="flex flex-row space-x-3">
        <IngestionPointsCard ingestionPoints={data.infrastructure.ingestionPoints}/>
        <QueuesCard queues={data.infrastructure.queues}/>
        <DatabasesCard databases={data.infrastructure.databases}/>
      </div>
    </section>    
  );
}