import { Card, CardContent, CardHeader } from "@/components/ui/card";
import TabNav from "./tab-nav";

interface Props {
  filterCard: React.ReactElement;
  table: React.ReactElement;
  chart: React.ReactElement;
}
export default function ReportLayout({ filterCard, table, chart }: Props) {
  return (
    <div className="grid grid-rows-4 grid-cols-5 gap-4 h-screen w-full p-4">
      <Card className="row-span-2 col-span-2 overflow-auto">
        <CardHeader className="sticky top-0 bg-background">
          <TabNav />
        </CardHeader>
        <CardContent>{filterCard}</CardContent>
      </Card>
      <Card className="col-span-3 row-span-2">{chart}</Card>
      <Card className="row-span-2 col-span-5 overflow-auto">{table}</Card>
    </div>
  );
}
