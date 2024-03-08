import { getData } from "./data";
import { ChartPage } from "./chart-page";
import { createFunnelQuery } from "@/data/funnel-query";
import { eventTables } from "@/data/event-tables";

export default async function Home() {

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-4">
      <ChartPage />
    </main>
  );
}
