import getStarted from "/videos/Demo-video.mp4";
import { Section } from "design-system/components/containers";
import dynamic from "next/dynamic";
const Video = dynamic(() => import("next-video"), { ssr: false });

export const DemoSection = () => {
  return (
    <Section>
      <div className="bg-muted py-36 px-40 aspect-video">
        <Video src={getStarted} className="aspect-video" />
      </div>
    </Section>
  );
};
