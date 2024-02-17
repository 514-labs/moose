import { Primitive } from "app/mock";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "components/ui/card";

interface PrimitiveCardProps {
  primitive: Primitive;
}

export function PrimitiveCard({ primitive }: PrimitiveCardProps) {
  return (
    <Card className="grow basis-0">
      <CardHeader>
        <CardTitle>{primitive.name}</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex flex-row">
          <div className="flex flex-col space-y-1.5">
            <div className="flex flex-row space-x-1.5">
              <span className="text-muted-foreground">Version:</span>
              <span className="">{primitive.version}</span>
            </div>
          </div>
        </div>
      </CardContent>
      <CardFooter>
        <div className="flex flex-row">
          <div className="flex flex-row space-x-1.5">
            <span className="text-muted-foreground">Docs:</span>
            <span className="">{primitive.docLink}</span>
          </div>
        </div>
      </CardFooter>
    </Card>
  );
}
