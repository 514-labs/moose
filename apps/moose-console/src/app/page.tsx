
import { Separator } from "components/ui/separator";
import { Button } from "components/ui/button";
import Link from "next/link";
import { getCliData } from "./db";
import { unstable_noStore as noStore } from 'next/cache';
import { ChevronRight } from "lucide-react";

interface OverviewCardHeaderProps {
  numItems?: number;
  title: string;
  href: string;
}

function OverviewCardHeader({ numItems, title, href }: OverviewCardHeaderProps) {
  return (
    <div className="text-4xl py-4 flex flex-row">
      <div className="grow text-ellipsis text-nowrap">{numItems ? `${numItems} ${title}` : title}</div>
      <div className="flex-shrink-0">
        <Link href={href}>
          <Button className="border-primary" variant="link">
            <ChevronRight className="h-4 w-4" />
          </Button>
        </Link>
      </div>
    </div>
  )
}


export default async function Primitives(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  return (
    <section className="p-4">
      <div className="text-5xl py-10">Overview</div>
      <div className="mb-20">
        <div className="text-3xl py-6 text-muted-foreground">Primitives | Docs</div>
        <div className="grid grid-cols-3 gap-4">
          <div className="col-span-3 xl:col-span-1">
            <OverviewCardHeader title="Models" numItems={data.models.length} href="primitives/models" />
            <Separator />
            {data.models.slice(0, 10).map((model, index) => (
              <Link href={`/primitives/models/${model.name}`} key={index}>
                <div key={index} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="py-4 text-muted-foreground">{model.name}</div>
                  <Separator />
                </div>
              </Link>
            ))}
          </div>
          <div className="col-span-3 xl:col-span-1 flex flex-col">
            <OverviewCardHeader title="Flows" numItems={0} href="docs" />
            <Separator />
            <div className="py-4 grow flex flex-col justify-center">
              <div className=" text-muted-foreground">Flows are in active development. Read the docs or join our community to give your input or contribute</div>
              <div className="flex space-x-4">
                <Button variant="outline" className="mt-4">Join our community</Button>
                <Button variant="link" className="mt-4">Docs</Button>
              </div>
            </div>
            <Separator />
          </div>
          <div className="col-span-3 xl:col-span-1 flex flex-col">
            <OverviewCardHeader title="Insights" numItems={0} href="docs" />
            <Separator />
            <div className="py-4 grow flex flex-col justify-center">
              <div className="text-muted-foreground">Insights are coming soon. We&apos;re currently gathering feedback from our community.</div>
              <div className="flex space-x-4">
                <Button variant="outline" className="mt-4">Provide your input</Button>
              </div>
            </div>
            <Separator />
          </div>
        </div>
      </div>

      <div>
        <div className="text-3xl py-6 text-muted-foreground">Infrastructure | Docs</div>
        <div className="grid grid-cols-3 gap-4">
          <div className="col-span-3 xl:col-span-1">
            <OverviewCardHeader title="Ingestion points" numItems={data.ingestionPoints.length} href="infrastructure/ingestion-points" />
            <Separator />
            {data.ingestionPoints.slice(0, 10).map((ingestionPoint, index) => (
              <Link href={`/infrastructure/ingestion-points/${ingestionPoint.route_path.split("/").at(-1)}`} key={index}>
                <div key={index} className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="py-4 text-muted-foreground">{ingestionPoint.route_path}</div>
                  <Separator />
                </div>
              </Link>
            ))}
          </div>
          <div className="col-span-3 xl:col-span-1">
            <OverviewCardHeader title="Tables" numItems={data.tables.filter(t => t.engine !== "MaterializedView" && !t.name.includes(".inner")).length} href="infrastructure/databases/tables?type=table" />
            <Separator />
            {data.tables.filter(t => t.engine !== "MaterializedView" && !t.name.includes(".inner")).slice(0, 10).map((table, index) => (
              <Link href={`/infrastructure/databases/${table.database}/tables/${table.uuid}`} key={index}>
                <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer" >
                  <div className="py-4 text-muted-foreground">{table.name}</div>
                  <Separator />
                </div>
              </Link>
            ))}
          </div>
          <div className="col-span-3 xl:col-span-1">
            <OverviewCardHeader title="Views" numItems={data.tables.filter(t => t.engine === "MaterializedView" && !t.name.includes(".inner")).length} href="infrastructure/databases/tables?type=view" />
            <Separator />
            {data.tables.filter(t => t.engine === "MaterializedView").slice(0, 10).map((table, index) => (
              <Link href={`/infrastructure/databases/${table.database}/tables/${table.uuid}`} key={index} >
                <div className="hover:bg-accent hover:text-accent-foreground hover:cursor-pointer">
                  <div className="py-4 text-muted-foreground">{table.name}</div>
                  <Separator />
                </div>
              </Link>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
