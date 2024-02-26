import Link from "next/link";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { ChevronRight } from "lucide-react";
import { Separator } from "./ui/separator";

interface OverviewItems {
  name: string;
  link: string;
}

interface OverviewCardProps {
  title: string;
  numItems: number;
  link: string;
  items: OverviewItems[];
}
export default function OverviewCard({
  title,
  numItems,
  link,
  items,
}: OverviewCardProps) {
  return (
    <Card className="h-full">
      <Link href={link}>
        <CardHeader className="">
          <CardTitle className="flex hover:text-white justify-between">
            <div>
              {numItems} {title}
            </div>{" "}
            <ChevronRight className="h-6 w-6" />
          </CardTitle>
        </CardHeader>
      </Link>
      <CardContent className="m-0 p-0">
        {items?.length ? (
          items.map((model, index) => (
            <Link href={model.link} key={index}>
              <Separator />
              <div
                key={index}
                className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer"
              >
                <div className="py-4 mx-6">{model.name}</div>
              </div>
            </Link>
          ))
        ) : (
          <div className="py-4 mx-6">No data</div>
        )}
      </CardContent>
    </Card>
  );
}
