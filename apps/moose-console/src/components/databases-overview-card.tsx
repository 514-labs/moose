import { Database } from "app/infrastructure/mock";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "components/ui/card";
import { Button, buttonVariants } from "./ui/button";
import { Separator } from "./ui/separator";

interface DatabaseCardProps {
  databases: Database[];
}

export function DatabasesCard({ databases }: DatabaseCardProps) {
  return (
    <Card className="grow basis-0">
      <CardHeader>
        <CardTitle>Databases</CardTitle>
        <CardDescription>
          These are the analytical databases that you use to store your data
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ul className="">
          {databases.map((database, index) => (
            <li key={index}>
              <div className="py-2 flex flex-row">
                <div>
                  <div>{database.name}</div>
                  <div className="text-muted-foreground">
                    {database.connectionURL}
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
