import React from "react";
import { RightsComponent } from "../../components/RightsComponent";
import { LogoComponent } from "../../components/LogoComponent";

export const FooterSection = () => {
  return (
    <div>
      <div className="flex sm:flex-row content-center justify-center grow flex-col gap-y-6 mt-6 p-10 sm:py-8">
        <RightsComponent />
        <LogoComponent />
      </div>
    </div>
  );
};
