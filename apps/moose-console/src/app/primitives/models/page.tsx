"use client";
import { Separator } from "components/ui/separator";
import { ModelsTable } from "components/models-table";
import { Card } from "components/ui/card";
import { NavBreadCrumb } from "components/nav-breadcrumb";
import { useContext } from "react";
import { VersionContext } from "version-context";

export default function ModelsPage() {
  const { models } = useContext(VersionContext);
  const modelMeta = models.map(({ model }) => model);

  return (
    <section className="p-4 max-h-screen grow overflow-y-auto">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">{models.length} Models</div>
        <div className="py-5 max-w-screen-md">
          Models define the shape of the data that your MooseJS app expects. If
          you want to learn more about them, head to the{" "}
          <a className="underline" href="">
            documentation
          </a>
        </div>
        <Separator />
        <Card>
          <ModelsTable models={modelMeta} />
        </Card>
      </div>
    </section>
  );
}
