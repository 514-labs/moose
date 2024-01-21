
import {
    Card,
    CardContent,
  } from "components/ui/card"
import { Separator } from "./ui/separator"
import { Button, buttonVariants } from "./ui/button"
import { IngestionPoint } from "app/infrastructure/mock"
  

interface IngestionPointsListProps {
    ingestionPoints: IngestionPoint[]
}

export function IngestionPointsListCard({ ingestionPoints }: IngestionPointsListProps) {
    return (
        <Card className="w-full">
            <CardContent className="p-0">
                <ul className="">
                    {ingestionPoints.map((model, index) => (
                        <li key={index}>
                            <div className="py-2 flex flex-row p-4">
                                <div>
                                    <div>{model.name}</div>
                                    <div className="text-muted-foreground">{model.description}</div>
                                </div>
                                <span className="flex-grow"/>
                                <div>
                                    <Button className={buttonVariants({ variant: "outline" })}>more</Button>
                                </div>
                            </div>
                            {index < ingestionPoints.length - 1 && <Separator/>}
                        </li>
                    ))}
                </ul>                
            </CardContent>
        </Card>
    )
}