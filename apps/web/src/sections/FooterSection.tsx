import React from "react";
import { RightsComponent } from "../app/components/RightsComponent";
import { LogoComponent } from "../app/components/LogoComponent";
import { AnimatedDescription } from "../app/components/AnimatedDescription";

export const FooterSection = () => {
  return (
    <div>
      <div className="flex sm:flex-row content-center justify-center grow flex-col gap-y-6 mt-6 p-10 sm:py-5">
        <RightsComponent />
        <LogoComponent />
      </div>
    </div>
  );
};
