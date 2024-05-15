/* eslint-disable turbo/no-undeclared-env-vars */
"use client";
import { getModelByName } from "lib/utils";
import {
  bashSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
  jsSnippet,
  pythonSnippet,
  rustSnippet,
} from "lib/snippets";
import { NavBreadCrumb } from "components/nav-breadcrumb";
import ModelView from "app/ModelView";
import { MooseObject } from "app/types";
import { useContext } from "react";
import { VersionContext } from "version-context";

export default function Page({ params }: { params: { modelName: string } }) {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  const { models, cliData } = useContext(VersionContext);

  const model = getModelByName(models, params.modelName);

  if (!model) {
    return <div>Model not found</div>;
  }

  const jsCodeSnippet = jsSnippet(cliData, model);
  const pythonCodeSnippet = pythonSnippet(cliData, model);
  const bashCodeSnippet = bashSnippet(cliData, model);
  const rustCodeSnippet = rustSnippet(cliData, model);
  const clickhouseJSCode = clickhouseJSSnippet(cliData, model);
  const clickhousePythonCode = clickhousePythonSnippet(cliData, model);

  return (
    <section className="p-4 max-h-screen overflow-y-auto grow">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">{model.model.name}</div>
      </div>
      <ModelView
        mooseObject={MooseObject.Model}
        model={model}
        cliData={cliData}
        jsSnippet={jsCodeSnippet}
        clickhouseJSSnippet={clickhouseJSCode}
        clickhousePythonSnippet={clickhousePythonCode}
        pythonSnippet={pythonCodeSnippet}
        bashSnippet={bashCodeSnippet}
        rustSnippet={rustCodeSnippet}
      />
    </section>
  );
}
