"use client";
import { NavBreadCrumb } from "components/nav-breadcrumb";
import { PreviewTable } from "components/preview-table";
import { Card } from "components/ui/card";
import { Separator } from "components/ui/separator";
import { useContext } from "react";
import { VersionContext } from "version-context";

export default async function FlowsPage(): Promise<JSX.Element> {
  const { models } = useContext(VersionContext);
  const rows = models
    .filter((model) => model.flows && model.flows.length > 0)
    .flatMap((model) =>
      model.flows.map((flow) => ({
        source: model.model.name,
        destination: flow,
      })),
    );

  return (
    <section className="p-4 max-h-screen overflow-y-auto">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">{rows.length} Flows</div>
        <div className="py-5 max-w-screen-md">
          Flows enable you to process your data as it moves through your MooseJS
          application. If you want to learn more about them, head to the{" "}
          <a
            className="underline"
            href="https://docs.moosejs.com/building/flows/intro"
          >
            documentation
          </a>
        </div>
        {rows.length > 0 && (
          <>
            <Separator />
            <Card>
              <PreviewTable rows={rows} />
            </Card>
          </>
        )}
      </div>
    </section>
  );
}
