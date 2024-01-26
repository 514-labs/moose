/* eslint-disable turbo/no-undeclared-env-vars */

import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";

import { unstable_noStore as noStore } from "next/cache";
import { Model, modelMock } from "../mock";
import { infrastructureMock } from "app/infrastructure/mock";
import { TablesListCard } from "components/table-list-card";
import { ViewsListCard } from "components/view-list-card";
import { QueuesListCard } from "components/queues-list-card";
import { IngestionPointsCard } from "components/ingestion-points-overview-card";
import { IngestionPointsListCard } from "components/ingestion-points-list-card";


async function getModel(modelId: string): Promise<Model> {
  try {
    return modelMock.models.find((q) => q.id === modelId);  
  } catch (error) {
    return null
  }
}



export default async function Page({
  params,
}: {
  params: {modelId: string};
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const model = await getModel(params.modelId);

  const infra = infrastructureMock;

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-10">
          <div className="text-6xl">{model.name}</div>
          <div className="text-muted-foreground">{model.description}</div>
        </div>
        <div className="flex flex-row space-x-3 ">
            <Tabs defaultValue="ingestionPoints" className="flex-grow">
              <TabsList>
                  <TabsTrigger value="ingestionPoints">Ingestion Points</TabsTrigger>
                  <TabsTrigger value="queues">Queues</TabsTrigger>
                  <TabsTrigger value="tables">Tables</TabsTrigger>
                  <TabsTrigger value="views">Views</TabsTrigger>
                  <TabsTrigger value="snippets">Snippets</TabsTrigger>
              </TabsList>
              <TabsContent value="ingestionPoints">
                <IngestionPointsListCard ingestionPoints={infra.ingestionPoints}/>
              </TabsContent>
              <TabsContent value="queues">
                <QueuesListCard queues={infra.queues}/>
              </TabsContent>
              <TabsContent value="tables">
                <TablesListCard tables={infra.databases.flatMap(x => x.tables.map((t) => ({databaseId : x.id, ...t})))}/>
              </TabsContent>
              <TabsContent value="views">
                <ViewsListCard views={infra.databases.flatMap(x => x.views.map((t) => ({databaseId : x.id, ...t})))}/>
              </TabsContent>
              <TabsContent value="snippets">
                <code className="font-mono">// Some code</code>
              </TabsContent>
          </Tabs>
         
        </div>
      </section>   
   
  );
}
