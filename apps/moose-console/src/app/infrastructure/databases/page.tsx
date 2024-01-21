

import { cn } from "lib/utils";
import { Table, View, infrastructureMock } from "../mock";
import Link from "next/link";

import {
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
  } from "components/ui/accordion"

  import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs"





  import {
    Card,
    CardContent,
  } from "components/ui/card"
import { Button, buttonVariants } from "components/ui/button";
import { Separator } from "components/ui/separator";
import { Badge, badgeVariants } from "components/ui/badge";
  

interface TablesListCardProps {
    tables: Table[]
}

export function TablesListCard({ tables }: TablesListCardProps) {
    return (
        <Card className="w-full">
            <CardContent className="p-0">
                <ul className="">
                    {tables.map((table, index) => (
                        <li key={index}>
                            <div className="py-2 flex flex-row p-4">
                                <div>
                                    <div className="text-xl">{table.name}</div>
                                    <div className="text-muted-foreground">{table.description}</div>
                                    <div className="space-x-1 py-2">
                                        {table.fields.map((field, index) => (
                                            <Badge className={badgeVariants({ variant: "secondary" })} key={index}>{field.name} | {field.type} </Badge>
                                        ))}
                                    </div>
                                </div>
                                <span className="flex-grow"/>
                                <div>
                                    <Badge className={badgeVariants({ variant: "secondary" })} key={index}>{table.rowCount.toLocaleString("en-us")} rows</Badge>
                                    <span className="px-2 mt-0.5"><Badge>Table</Badge></span>
                                    <Button className={buttonVariants({ variant: "outline" })}>more</Button>
                                </div>
                            </div>
                            {index < tables.length - 1 && <Separator/>}
                        </li>
                    ))}
                </ul>                
            </CardContent>
        </Card>
    )
}


interface ViewsListCardProps {
    views: View[]
}

export function ViewsListCard({ views }: ViewsListCardProps) {
    return (
        <Card className="w-full">
            <CardContent className="p-0">
                <ul className="">
                    {views.map((view, index) => (
                        <li key={index}>
                            <div className="py-2 flex flex-row p-4">
                                <div>
                                    <div className="text-xl">{view.name}</div>
                                    <div className="text-muted-foreground">{view.description}</div>
                                    <div className="space-x-1 py-2">
                                        {view.fields.map((field, index) => (
                                            <Badge className={badgeVariants({ variant: "secondary" })} key={index}>{field.name} | {field.type} </Badge>
                                        ))}
                                    </div>
                                </div>
                                <span className="flex-grow"/>
                                <div>
                                    <Badge className={badgeVariants({ variant: "secondary" })} key={index}>{view.rowCount.toLocaleString("en-us")} rows</Badge>
                                    <span className="px-2 mt-0.5"><Badge>View</Badge></span>
                                    <Button className={buttonVariants({ variant: "outline" })}>more</Button>
                                </div>
                            </div>
                            {index < views.length - 1 && <Separator/>}
                        </li>
                    ))}
                </ul>                
            </CardContent>
        </Card>
    )
}



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