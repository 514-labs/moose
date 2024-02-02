import { Route, getCliData } from "app/db";
import { tabListStyle, tabTriggerStyle } from "components/style-utils";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { cn } from "lib/utils";

import { unstable_noStore as noStore } from "next/cache";
import Link from "next/link";


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
          <div className="text-6xl">
            <Link className="text-muted-foreground" href={"/"}>../</Link>
            <Link className="text-muted-foreground" href={"/infrastructure/ingestion-points"}>ingestion-points/</Link>
            {ingestionPoint.table_name}
          </div>
          <div className="text-muted-foreground py-4">{ingestionPoint.route_path}</div>
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
