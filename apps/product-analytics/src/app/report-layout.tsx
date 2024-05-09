import { Card, CardContent, CardHeader } from "@/components/ui/card";
import TabNav from "./tab-nav";

interface Props {
  filterCard: React.ReactElement;
  table: React.ReactElement;
  chart: React.ReactElement;
}
export default function ReportLayout({ filterCard, table, chart }: Props) {
  return (
    <div className="p-5 w-screen grid gap-5 grid-cols-3 overflow-auto p-0 w-full">
      <Card className="h-full flex flex-col rounded-2xl w-full">
        <CardHeader className="p-4 gap-4 w-full">
          <TabNav />
        </CardHeader>
        <CardContent>{filterCard}</CardContent>
      </Card>
      <Card className="p-4 col-span-3 w-full rounded-2xl">{chart}</Card>
      <Card className="p-4 row-span-2 col-span-4 overflow-auto w-full rounded-2xl">
        {table}
      </Card>
    </div>
  );
}
