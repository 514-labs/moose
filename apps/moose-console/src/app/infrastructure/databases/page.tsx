

import { cn } from "lib/utils";
import { infrastructureMock } from "../mock";
import { getCliData } from "app/db";
import { unstable_noStore as noStore } from "next/cache";

import {
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
  } from "components/ui/accordion"

import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs"

import { TablesListCard } from "components/table-list-card";
import { ViewsListCard } from "components/view-list-card";
  

export default async function DatabasesPage(): Promise<JSX.Element> {
    const data = infrastructureMock;
  
    return (
      <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-20">
            <div className="text-9xl">Databases</div>
            <div className="text-muted-foreground py-4">Databases store your data and provide a set of guarantees</div>
        </div>
        
        {data.databases.map((db, index) => (
            <Accordion key={index} type="multiple" defaultValue={["item-0"]}>
            <AccordionItem value={`item-${index}`}  className="border-b-0" >
                    <AccordionTrigger className="hover:no-underline">
                        <div className="text-start">
                            <div className="text-6xl hover:underline">{db.name}</div>
                            <div className="text-muted-foreground font-normal py-2">{db.description}</div>
                        </div>
                    </AccordionTrigger>
                    <AccordionContent className="font-normal">
                        <Tabs defaultValue="views">
                            <TabsList>
                                <TabsTrigger value="tables">Tables</TabsTrigger>
                                <TabsTrigger value="views">Views</TabsTrigger>
                            </TabsList>
                            <TabsContent value="tables">
                                <TablesListCard tables={db.tables}/>
                            </TabsContent>
                            <TabsContent value="views">
                                <ViewsListCard views={db.views}/>
                            </TabsContent>
                        </Tabs>
                    </AccordionContent>
                </AccordionItem>
            </Accordion>
        ))}

        
        
      </section>    
    );
  }