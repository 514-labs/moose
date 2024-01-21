import { infrastructureMock } from "app/infrastructure/mock";
import { IngestionPointsListCard } from "components/ingestion-points-list-card";

export default async function IngestionPointsPage(): Promise<JSX.Element> {
    const data = infrastructureMock;
  
    return (
      <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-20">
            <div className="text-9xl">Ingestion Points</div>
            <div className="text-muted-foreground py-5">Ingestion points help you scalably capture data from any source</div>
        </div>
        <IngestionPointsListCard ingestionPoints={data.ingestionPoints}/>
      </section>    
    );
  }