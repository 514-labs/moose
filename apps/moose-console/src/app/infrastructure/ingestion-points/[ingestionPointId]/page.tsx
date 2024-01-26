import { IngestionPoint, Queue, infrastructureMock } from "app/infrastructure/mock";
import { Field } from "app/mock";
import { Badge, badgeVariants } from "components/ui/badge";
import { Button, buttonVariants } from "components/ui/button";
import { Card, CardContent } from "components/ui/card";
import { Separator } from "components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { cn } from "lib/utils";

import { unstable_noStore as noStore } from "next/cache";


async function getIngestionPoint(ingestionPointId: string): Promise<IngestionPoint> {


  try {
    return infrastructureMock.ingestionPoints.find((ip) => ip.id === ingestionPointId);  
  } catch (error) {
    return null
  }
  
}




export default async function Page({
  params,
}: {
  params: {ingestionPointId: string};
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const queue = await getIngestionPoint(params.ingestionPointId);

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-10">
          <div className="text-6xl">{queue.name}</div>
          <div className="text-muted-foreground">{queue.description}</div>
        </div>
        <div className="flex flex-row space-x-3 ">
            <Tabs defaultValue="snippets" className="flex-grow">
              <TabsList>
                  <TabsTrigger value="snippets">Snippets</TabsTrigger>
                  <TabsTrigger value="logs">Logs</TabsTrigger>
              </TabsList>
              <TabsContent value="snippets">
               
              </TabsContent>
              <TabsContent value="logs">
                  
              </TabsContent>
              
              
          </Tabs>
         
        </div>
      </section>  
   
  );
}
