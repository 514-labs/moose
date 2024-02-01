import { Route, getCliData } from "app/db";
import { IngestionPoint, Queue, infrastructureMock } from "app/infrastructure/mock";
import { Field } from "app/mock";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";
import { Badge, badgeVariants } from "components/ui/badge";
import { Button, buttonVariants } from "components/ui/button";
import { Card, CardContent } from "components/ui/card";
import { Separator } from "components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { cn } from "lib/utils";

import { unstable_noStore as noStore } from "next/cache";


async function getIngestionPoint(ingestionPointId: string): Promise<Route> {
  try {
    const data = await getCliData();
    return data.ingestionPoints.find((ingestionPoint) => ingestionPoint.route_path.split("/").at(-1) === ingestionPointId);
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

  const ingestionPoint = await getIngestionPoint(params.ingestionPointId);

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-10">
          <div className="text-6xl">{ingestionPoint.route_path}</div>
          <div className="text-muted-foreground">{ingestionPoint.file_path}</div>
        </div>
        <div className="flex flex-row space-x-3 ">
            <Tabs defaultValue="snippets" className="flex-grow">
              <TabsList className={cn(tabListStyle)}>
                  <TabsTrigger className={cn(tabTriggerStyle)} value="snippets">Snippets</TabsTrigger>
                  <TabsTrigger className={cn(tabTriggerStyle)} value="logs">Logs</TabsTrigger>
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
