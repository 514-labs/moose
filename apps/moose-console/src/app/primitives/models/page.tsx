import { modelMock } from "./mock";
import Link from "next/link";
import { Separator } from "components/ui/separator";



export default async function ModelsPage(): Promise<JSX.Element> {
    const data = modelMock;
  
    return (
      <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-10">
            <div className="text-6xl">
              <Link className="text-muted-foreground" href="/primitives"> Primitives </Link>
              /
              <Link href="/primitives/models">{data.models.length} Models </Link>
            </div>
            <div className="text-muted-foreground py-5 max-w-screen-md">
              Models define the shape of the data that your MooseJS app expects. 
              If you want to learn more about them, head to the <a className="underline" href="">documentation</a>
            </div>
            <Separator />
            {data.models.map((model, index) => (
              <Link key={index} href={`/primitives/models/${model.id}`} >
                <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="py-4 text-muted-foreground">{model.name}</div>
                  <Separator />
                </div>
              </Link>
            ))}
        </div>
      </section>    
    );
  }