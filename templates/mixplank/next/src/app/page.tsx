import { loadMixPanelData } from "@/lib/load-mixpanel-data";
import { redirect } from "next/navigation";

export default async function Home() {
  redirect('/insights')
}
