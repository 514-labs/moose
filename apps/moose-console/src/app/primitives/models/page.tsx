import { ModelsCard } from "components/models-card";
import { modelMock } from "./mock";



export default async function ModelsPage(): Promise<JSX.Element> {
    const data = modelMock;
  
    return (
      <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-20">
            <div className="text-9xl">Models</div>
            <div className="text-muted-foreground py-5">Models define the shape of the data that your MooseJS app expects</div>
        </div>
        <ModelsCard models={data.models} />
        
        
      </section>    
    );
  }