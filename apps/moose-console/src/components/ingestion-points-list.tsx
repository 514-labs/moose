import { Route } from "app/db"
import { Separator } from "./ui/separator"
import Link from "next/link"

interface IngestionPointsListProps {
    ingestionPoints: Route[]
}

export function IngestionPointsList({ ingestionPoints }: IngestionPointsListProps) {
    return (
        <div className="">
            <Separator />
            {ingestionPoints.map((ingestionPoint, index) => (
                <Link key={index} href={`/infrastructure/ingestion-points/${ingestionPoint.table_name}`} >
                <div key={index} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer"> 
                    <div className="py-2 flex flex-row">
                        <div>
                            <div>{ingestionPoint.table_name}</div>
                            <div className="text-muted-foreground">{ingestionPoint.route_path}</div>
                        </div>
                        <span className="flex-grow"/>
                    </div>
                    <Separator/>
                </div>
                </Link>
            ))}
        </div>                
    )
}