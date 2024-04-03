"use client";
import { getModelByIngestionPointId } from "lib/utils";

import {
  jsSnippet,
  pythonSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
  bashSnippet,
} from "lib/snippets";
import { NavBreadCrumb } from "components/nav-breadcrumb";
import ModelView from "app/ModelView";
import { MooseObject } from "app/types";
import { useContext } from "react";
import { VersionContext } from "version-context";

export default function Page({
  params,
}: {
  params: { ingestionPointId: string };
}) {
  const { models, cliData } = useContext(VersionContext);

  const model = getModelByIngestionPointId(models, params.ingestionPointId);

  const { ingestion_point } = model;

  if (!ingestion_point) {
    return <div>Ingestion Point not found</div>;
  }

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">{ingestion_point.route_path}</div>
      </div>
      <ModelView
        mooseObject={MooseObject.IngestionPoint}
        model={model}
        cliData={cliData}
        bashSnippet={bashSnippet(cliData, model)}
        jsSnippet={jsSnippet(cliData, model)}
        pythonSnippet={pythonSnippet(cliData, model)}
        clickhouseJSSnippet={clickhouseJSSnippet(cliData, model)}
        clickhousePythonSnippet={clickhousePythonSnippet(cliData, model)}
      />
    </section>
  );
}
