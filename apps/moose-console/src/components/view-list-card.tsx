import { View } from "../app/infrastructure/mock";
import { Card, CardContent } from "components/ui/card";
import { Button, buttonVariants } from "components/ui/button";
import { Separator } from "components/ui/separator";
import { Badge, badgeVariants } from "components/ui/badge";

interface ViewsListCardProps {
  views: View[];
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
                  <div className="text-muted-foreground">
                    {view.description}
                  </div>
                  <div className="space-x-1 py-2">
                    {view.fields.map((field, index) => (
                      <Badge
                        className={badgeVariants({ variant: "secondary" })}
                        key={index}
                      >
                        {field.name} | {field.type}{" "}
                      </Badge>
                    ))}
                  </div>
                </div>
                <span className="flex-grow" />
                <div>
                  <Badge
                    className={badgeVariants({ variant: "secondary" })}
                    key={index}
                  >
                    {view.rowCount.toLocaleString("en-us")} rows
                  </Badge>
                  <span className="px-2 mt-0.5">
                    <Badge>Clickhouse View</Badge>
                  </span>
                  <Button className={buttonVariants({ variant: "outline" })}>
                    more
                  </Button>
                </div>
              </div>
              {index < views.length - 1 && <Separator />}
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  );
}
