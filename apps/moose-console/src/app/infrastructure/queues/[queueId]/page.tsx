/* eslint-disable turbo/no-undeclared-env-vars */

import { BaseResultSet, createClient } from "@clickhouse/client-web";
import { Queue, Row, Value, infrastructureMock } from "app/infrastructure/mock";
import { Field } from "app/mock";
import { Badge, badgeVariants } from "components/ui/badge";
import { Button, buttonVariants } from "components/ui/button";
import { Card, CardContent } from "components/ui/card";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "components/ui/resizable";
import { Separator } from "components/ui/separator";
import { Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow } from "components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { cn } from "lib/utils";

import { unstable_noStore as noStore } from "next/cache";


async function getQueue(queueId: string): Promise<Queue> {


  try {
    return infrastructureMock.queues.find((q) => q.id === queueId);  
  } catch (error) {
    return null
  }
  
}



export default async function Page({
  params,
}: {
  params: {queueId: string};
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const queue = await getQueue(params.queueId);

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-10">
          <div className="text-6xl">{queue.name}</div>
          <div className="text-muted-foreground">{queue.description}</div>
        </div>
        <div className="flex flex-row space-x-3 ">
            <Tabs defaultValue="logs" className="flex-grow">
              <TabsList>
                  <TabsTrigger value="logs">Logs</TabsTrigger>
                  <TabsTrigger value="snippets">Snippets</TabsTrigger>
              </TabsList>
              <TabsContent value="logs">
                  
              </TabsContent>
              <TabsContent value="snippets">
               
              </TabsContent>
              
          </Tabs>
         
        </div>
      </section>  
   
  );
}
