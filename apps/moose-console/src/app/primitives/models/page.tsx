import { Separator } from "components/ui/separator";
import { getCliData } from "app/db";
import { unstable_noStore as noStore } from "next/cache";
import { ModelsTable } from "components/models-table";
import { Card } from "components/ui/card";
import { NavBreadCrumb } from "components/nav-breadcrumb";

export default async function ModelsPage(): Promise<JSX.Element> {
  noStore();
  const data = await getCliData();
  const models = data.current.models.map(({ model }) => model);

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
          <ModelsTable models={models} />
        </Card>
      </div>
    </section>
  );
}
