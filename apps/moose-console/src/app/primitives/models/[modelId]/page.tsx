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
import Link from "next/link";
import { Separator } from "components/ui/separator";
import { cn } from "lib/utils";


async function getModel(modelId: string): Promise<Model> {
  try {
    return modelMock.models.find((q) => q.id === modelId);
  } catch (error) {
    return null
  }
}

const triggerStyle = "rounded-none px-3 py-1.5 text-md font-normal ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:border-b-2 data-[state=active]:text-foreground data-[state=active]:shadow-none data-[state=active]:border-white"



export default async function Page({
  params,
}: {
  params: { modelId: string };
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const model = await getModel(params.modelId);

  const infra = infrastructureMock;

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
      <div className="py-10">
        <div className="text-6xl">
          <Link className="text-muted-foreground" href="/primitives"> ... / </Link>
          <Link className="text-muted-foreground" href="/primitives/models"> Models </Link>
          <Link href="/primitives/models">/ {model.name} </Link>
        </div>
        <div className="text-muted-foreground py-5 max-w-screen-md">
          Models define the shape of the data that your MooseJS app expects.
          If you want to learn more about them, head to the <a className="underline" href="">documentation</a>
        </div>

        <div className="flex flex-row space-x-3 ">
          <Tabs defaultValue="ingestionPoints" className="flex-grow">
            <TabsList className="inline-flex h-10 items-center justify-center rounded-none bg-transparent p-0 text-muted-foreground -mb-0.5">
              <TabsTrigger className={cn(triggerStyle)} value="ingestionPoints">Ingestion Points</TabsTrigger>
              <TabsTrigger className={cn(triggerStyle)} value="queues">Queues</TabsTrigger>
              <TabsTrigger className={cn(triggerStyle)} value="tables">Tables</TabsTrigger>
              <TabsTrigger className={cn(triggerStyle)} value="views">Views</TabsTrigger>
              <TabsTrigger className={cn(triggerStyle)} value="snippets">Snippets</TabsTrigger>
            </TabsList>
            <Separator />
            <TabsContent value="ingestionPoints">
              {infra.ingestionPoints.map((ingestionPoint) => (
                <Link href={`/infrastructure/ingestion-points/${ingestionPoint.id}`} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="py-4 text-muted-foreground">{ingestionPoint.name}</div>
                  <Separator />
                </Link>
              ))}
            </TabsContent>
            <TabsContent value="queues">
              {infra.ingestionPoints.map((queue) => (
                <Link href={`/infrastructure/queues/${queue.id}`} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="py-4 text-muted-foreground">{queue.name}</div>
                  <Separator />
                </Link>
              ))}
            </TabsContent>
            <TabsContent value="tables">
              {infra.databases.flatMap(x => x.tables.map((t) => ({ databaseId: x.id, ...t }))).map((table) => (
                <Link href={`/infrastructure/databases/${table.databaseId}/tables/${table.id}`} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="py-4 text-muted-foreground">{table.name}</div>
                  <Separator />
                </Link>
              ))}
            </TabsContent>
            <TabsContent value="views">
              {infra.databases.flatMap(x => x.views.map((t) => ({ databaseId: x.id, ...t }))).map((view) => (
                <Link href={`/infrastructure/databases/${view.databaseId}/views/${view.id}`} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="py-4 text-muted-foreground">{view.name}</div>
                  <Separator />
                </Link>
              ))}

            </TabsContent>
            <TabsContent value="snippets">
              <code className="font-mono">// Some code</code>
            </TabsContent>
          </Tabs>

        </div>
      </div>
    </section>

  );
}
