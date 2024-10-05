import { TrackButton } from "@514labs/design-system-components/components";
import React from "react";
import Link from "next/link";
import { cn } from "@514labs/design-system-components/utils";

export const HeroSection = ({ className }: { className?: string }) => {
  const content = {
    title: "Prototype & scale data-intensive applications without the hassle",
    description:
      "An open source developer framework for your data & analytics stack",
    primaryCTA: "Get started",
    primaryCTAUrl: "https://docs.getmoose.dev/quickstart",
    secondaryCTA: "View Docs",
    secondaryCTAUrl: "https://docs.getmoose.dev",
  };

  return (
    <section className={cn(className)}>
      <div className=" text-4xl sm:text-6xl 2xl:text-7xl text-center">
        {content.title}
      </div>
      <div className="text-2xl 2xl:text-4xl text-center text-muted-foreground py-5">
        {content.description}
      </div>

      <div className="flex flex-col max-md:w-full max-md:space-y-5 md:flex-row md:space-x-5">
        <Link className="grow flex-1" href={content.primaryCTAUrl}>
          <TrackButton
            name="hero-primary-cta"
            subject={content.primaryCTA}
            targetUrl={content.primaryCTAUrl}
            className="w-full"
            size="lg"
          >
            {content.primaryCTA}
          </TrackButton>
        </Link>
        <Link className="max-md:grow flex-1" href={content.secondaryCTAUrl}>
          <TrackButton
            name="hero-seconday-cta"
            subject={content.secondaryCTA}
            targetUrl={content.secondaryCTAUrl}
            className="w-full"
            size="lg"
            variant="outline"
          >
            {content.secondaryCTA}
          </TrackButton>
        </Link>
      </div>
    </section>
  );
};
// export const HeroSection = () => {

//   return (
//     <Fragment>
//       <Section className="max-w-5xl mx-auto flex flex-col items-center p-0 p-20 px-5">
//         <Grid>
//           <FullWidthContentContainer>
//             <div className=" text-4xl sm:text-6xl 2xl:text-7xl text-center">
//               {content.tagLine}
//             </div>
//             <div className="text-2xl 2xl:text-4xl text-center text-muted-foreground py-5">
//               {content.description}
//             </div>
//             <div className="flex gap-5">
//               {content.ctas.map((cta, index) => (
//                 <Link key={index} href={cta.href}>
//                   <TrackButton
//                     name={`Hero CTA ${cta.label}`}
//                     subject={content.tagLine}
//                     targetUrl={cta.href}
//                     variant={cta.variant as "default" | "outline"}
//                     size="lg"
//                   >
//                     {cta.label}
//                   </TrackButton>
//                 </Link>
//               ))}
//             </div>
//           </FullWidthContentContainer>
//         </Grid>
//       </Section>
//       <ValuePropHeroSection />
//     </Fragment>
//   );
// };
