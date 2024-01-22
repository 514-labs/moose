import { Queue } from "app/infrastructure/mock"
import { Card, CardContent } from "./ui/card"
import { Badge, badgeVariants } from "./ui/badge"
import { Button, buttonVariants } from "./ui/button"
import { Separator } from "./ui/separator"

interface QueuesListCardProps {
    queues: Queue[]
}

export function QueuesListCard({ queues }: QueuesListCardProps) {
    return (
        <Card className="w-full">
            <CardContent className="p-0">
                <ul className="">
                    {queues.map((queue, index) => (
                        <li key={index}>
                            <div className="py-2 flex flex-row p-4">
                                <div>
                                    <div className="text-xl">{queue.name}</div>
                                    <div className="text-muted-foreground">{queue.description}</div>
                                </div>
                                <span className="flex-grow"/>
                                <div>
                                    <Badge className={badgeVariants({ variant: "secondary" })} key={index}>{queue.messageCount.toLocaleString("en-us")} messages</Badge>
                                    <span className="px-2 mt-0.5"><Badge>Redpanda Topic</Badge></span>
                                    <Button className={buttonVariants({ variant: "outline" })}>more</Button>
                                </div>
                            </div>
                            {index < queues.length - 1 && <Separator/>}
                        </li>
                    ))}
                </ul>                
            </CardContent>
        </Card>
    )
}