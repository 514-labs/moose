/* eslint-disable turbo/no-undeclared-env-vars */

import { unstable_noStore as noStore } from "next/cache";
import { getRelatedInfra } from "lib/utils";
import { CliData, DataModel, getCliData } from "app/db";
import {
  bashSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
  jsSnippet,
  pythonSnippet,
} from "lib/snippets";
import { NavBreadCrumb } from "components/nav-breadcrumb";
import ModelView from "app/ModelView";

async function getModel(name: string, data: CliData): Promise<DataModel> {
  try {
    return data.models.find((x) => x.name === name);
  } catch (error) {
    return null;
  }
}

export default async function Page({
  params,
}: {
  params: { modelName: string };
}): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();

  const data = await getCliData();
  const model = await getModel(params.modelName, data);
  const infra = getRelatedInfra(model, data, model);
  const triggerTable = infra.tables.find(
    (t) => t.name.includes(model.name) && t.engine === "MergeTree"
  );

  const jsCodeSnippet = jsSnippet(data, model);
  const pythonCodeSnippet = pythonSnippet(data, model);
  const bashCodeSnippet = bashSnippet(data, model);
  const clickhouseJSCode = clickhouseJSSnippet(data, model);
  const clickhousePythonCode = clickhousePythonSnippet(data, model);

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">{model.name}</div>
      </div>
      <ModelView
        table={triggerTable}
        cliData={data}
        jsSnippet={jsCodeSnippet}
        clickhouseJSSnippet={clickhouseJSCode}
        clickhousePythonSnippet={clickhousePythonCode}
        pythonSnippet={pythonCodeSnippet}
        bashSnippet={bashCodeSnippet}
      />
    </section>
  );
}
