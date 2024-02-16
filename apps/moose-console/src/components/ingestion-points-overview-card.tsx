import { IngestionPoint } from "app/infrastructure/mock";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "components/ui/card";
import { Separator } from "./ui/separator";
import { Button, buttonVariants } from "./ui/button";

interface IngestionPointsCardProps {
  ingestionPoints: IngestionPoint[];
}

export function IngestionPointsCard({
  ingestionPoints,
}: IngestionPointsCardProps) {
  return (
    <Card className="grow basis-0">
      <CardHeader>
        <CardTitle>Ingestion Points</CardTitle>
        <CardDescription>
          These are your data capture ingestion points where you can send data
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ul className="">
          {ingestionPoints.map((ingestionPoint, index) => (
            <li key={index}>
              <div className="py-2 flex flex-row">
                <div>
                  <div>{ingestionPoint.name}</div>
                  <div className="text-muted-foreground">
                    {ingestionPoint.connectionURL}
                  </div>
                </div>
                <span className="flex-grow" />
                <div>
                  <Button className={buttonVariants({ variant: "outline" })}>
                    more
                  </Button>
                </div>
              </div>
              <Separator />
            </li>
          ))}
        </ul>
      </CardContent>
      <CardFooter></CardFooter>
    </Card>
  );
}
