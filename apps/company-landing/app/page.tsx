// import { EmailSection } from "./sections/EmailSection";
import { FooterSection } from "./sections/FooterSection";

import { ManifestoSection } from "./sections/home/manifesto-section";

export default function Home() {
  return (
    <main className="min-h-screen w-full relative mx-auto xl:max-w-screen-xl">
      <ManifestoSection />
      <FooterSection />
      {/* <EmailSection /> */}
    </main>
  );
}
