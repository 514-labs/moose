import { Route, getCliData } from "app/db";
import { getModelFromRoute } from "lib/utils";

import { unstable_noStore as noStore } from "next/cache";
import Link from "next/link";
import IngestionPointTabs from "./ingestion-point-tabs";
import { jsSnippet, pythonSnippet, clickhouseJSSnippet, clickhousePythonSnippet } from "lib/snippets";


async function getIngestionPoint(ingestionPointId: string): Promise<Route> {
  try {
    const data = await getCliData();
    return data.ingestionPoints.find((ingestionPoint) => ingestionPoint.route_path.split("/").at(-1) === ingestionPointId);
  } catch (error) {
    return null
  }
  
}

export default async function Page({
  params,
}: {
  params: {ingestionPointId: string};
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const ingestionPoint = await getIngestionPoint(params.ingestionPointId);
  const cliData = await getCliData();
  const model = getModelFromRoute(ingestionPoint, cliData);

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
        <div className="py-10">
          <div className="text-6xl">
            <Link className="text-muted-foreground" href={"/"}>../</Link>
            <Link className="text-muted-foreground" href={"/infrastructure/ingestion-points"}>ingestion-points/</Link>
            {ingestionPoint.table_name}
          </div>
          <div className="text-muted-foreground py-4">{ingestionPoint.route_path}</div>
        </div>
        <div className="space-x-3 flex-grow">
            <IngestionPointTabs 
              ingestionPoint={ingestionPoint} 
              cliData={cliData} 
              jsSnippet={jsSnippet(cliData, model)} 
              pythonSnippet={pythonSnippet(cliData, model)} 
              clickhouseJSSnippet={clickhouseJSSnippet(cliData, model)} 
              clickhousePythonSnippet={clickhousePythonSnippet(cliData, model)} />
        </div>
      </section>  
   
  );
}
