import { QueuesListCard } from "components/queues-list-card";
import { infrastructureMock } from "../mock";




export default async function Primitives(): Promise<JSX.Element> {
    const data = infrastructureMock;
  
    return (
      <section className="p-4 max-h-screen overflow-y-auto">
        <div className="text-9xl py-20">Queues</div>
        <div className="flex flex-row space-x-3">
            <QueuesListCard queues={infrastructureMock.queues} />
         
        </div>
      </section>    
    );
  }