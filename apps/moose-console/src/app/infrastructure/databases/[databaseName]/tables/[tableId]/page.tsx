/* eslint-disable turbo/no-undeclared-env-vars */
"use client";
import Link from "next/link";
import {
  bashSnippet,
  clickhouseJSSnippet,
  clickhousePythonSnippet,
  jsSnippet,
  pythonSnippet,
} from "lib/snippets";
import { getModelByTableId, tableIsView } from "lib/utils";
import { Fragment, useContext } from "react";
import ModelView from "app/ModelView";
import { MooseObject } from "app/types";
import { VersionContext } from "version-context";
import { useTrackPageView } from "app/trackable-components";

export default function Page({
  params,
}: {
  params: { databaseName: string; tableId: string };
  searchParams: { tab: string };
}) {
  useTrackPageView();
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  const { models, cliData } = useContext(VersionContext);
  if (models.length === 0) {
    return <div>Table not found</div>;
  }
  const model = getModelByTableId(models, params.tableId);

  const isView = tableIsView(model.table);

  return (
    <section className="p-4 max-h-screen flex-grow overflow-y-auto flex flex-col grow">
      <div className="text-base text-muted-foreground flex">
        <Fragment>
          <Link className={`capitalize text-white`} href={"/infrastructure"}>
            Infrastructure
          </Link>
          <div className="px-1">/</div>
          <Link className={`capitalize text-white`} href={"/infrastructure"}>
            Tables
          </Link>
        </Fragment>
      </div>
      <div className="py-10">
        <div className="text-8xl">{model.table.name}</div>
        <div className="text-muted-foreground">{model.table.engine}</div>
      </div>
      <ModelView
        model={model}
        mooseObject={isView ? MooseObject.View : MooseObject.Table}
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
