
import { Queue } from "app/infrastructure/mock"
import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle,
  } from "components/ui/card"
import { Button, buttonVariants } from "./ui/button"
import { Separator } from "./ui/separator"
  

interface QueuesCardProps {
    queues: Queue[]
}

export function QueuesCard({ queues }: QueuesCardProps) {
    return (
        <Card className="grow basis-0">
            <CardHeader>
                <CardTitle>Queues</CardTitle>
                <CardDescription>Queues ensure that your data can reliably get to your databases under any load</CardDescription>
            </CardHeader>
            <CardContent>
                <ul className="">
                    {queues.map((queue, index) => (
                        <li key={index}>
                            <div className="py-2 flex flex-row">
                                <div>
                                    <div>{queue.name}</div>
                                    <div>{queue.connectionUrl}</div>
                                </div>
                                <span className="flex-grow"/>
                                <div>
                                    <Button className={buttonVariants({ variant: "outline" })}>more</Button>
                                </div>
                            </div>
                            <Separator/>
                        </li>
                    ))}
                </ul>    
            </CardContent>
            <CardFooter>
                
            </CardFooter>
        </Card>
    )
}