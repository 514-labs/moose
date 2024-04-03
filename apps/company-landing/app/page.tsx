import { sendServerEvent } from "event-capture/server-event";
import { EmailSection } from "./sections/EmailSection";
import { FooterSection } from "./sections/FooterSection";

import { ManifestoSection } from "./sections/home/manifesto-section";

export default function Home() {
  sendServerEvent("page_view", { page: "home" });
  return (
    <main className="min-h-screen">
      <ManifestoSection />
      <FooterSection />
      <EmailSection />
    </main>
  );
}
