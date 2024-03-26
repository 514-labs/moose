import { Route, getCliData } from "app/db";
import { getModelFromRoute } from "lib/utils";

import { unstable_noStore as noStore } from "next/cache";
import IngestionPointTabs from "./ingestion-point-tabs";
import {
  jsSnippet,
  pythonSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
  bashSnippet,
} from "lib/snippets";
import { NavBreadCrumb } from "components/nav-breadcrumb";

async function getIngestionPoint(
  ingestionPointId: string,
): Promise<Route | undefined> {
  const data = await getCliData();
  return data.ingestionPoints.find(
    (ingestionPoint) =>
      ingestionPoint.route_path.split("/").at(-1) === ingestionPointId,
  );
}

export default async function Page({
  params,
}: {
  params: { ingestionPointId: string };
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const ingestionPoint = await getIngestionPoint(params.ingestionPointId);
  const cliData = await getCliData();

  if (!ingestionPoint) {
    return <div>Ingestion Point not found</div>;
  }

  const model = getModelFromRoute(ingestionPoint, cliData);

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">{ingestionPoint.route_path}</div>
      </div>
      <div className="space-x-3 flex-grow">
        <IngestionPointTabs
          ingestionPoint={ingestionPoint}
          cliData={cliData}
          bashSnippet={bashSnippet(cliData, model)}
          jsSnippet={jsSnippet(cliData, model)}
          pythonSnippet={pythonSnippet(cliData, model)}
          clickhouseJSSnippet={clickhouseJSSnippet(cliData, model)}
          clickhousePythonSnippet={clickhousePythonSnippet(cliData, model)}
        />
      </div>
    </section>
  );
}
