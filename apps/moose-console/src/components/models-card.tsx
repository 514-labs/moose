
import {
    Card,
    CardContent,
  } from "components/ui/card"
import { Separator } from "./ui/separator"
import { Model } from "app/primitives/models/mock"
import { ChevronRightButton } from "./chevron-right-button"
  

interface ModelsCardProps {
    models: Model[]
}

export function ModelsCard({ models }: ModelsCardProps) {
    return (
        <Card className="w-full">
            <CardContent className="p-0">
                <ul className="">
                    {models.map((model, index) => (
                        <li key={index}>
                            <div className="py-2 flex flex-row p-4">
                                <div>
                                    <div>{model.name}</div>
                                    <div className="text-muted-foreground">{model.description}</div>
                                </div>
                                <span className="flex-grow"/>
                                <div>
                                    <ChevronRightButton href={`/primitives/models/${model.id}`}/>
                                </div>
                            </div>
                            {index < models.length - 1 && <Separator/>}
                        </li>
                    ))}
                </ul>                
            </CardContent>
        </Card>
    )
}