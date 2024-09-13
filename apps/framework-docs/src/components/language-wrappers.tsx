import React, { ReactNode } from "react";
import { useLanguage } from "./LanguageContext";
import { cn } from "@514labs/design-system-components/utils";

interface LanguageProps {
  children: ReactNode;
}

export const TypeScript: React.FC<LanguageProps> = ({ children }) => {
  const { language } = useLanguage();
  return (
    <div className={cn(language === "typescript" ? "" : "hidden")}>
      {children}
    </div>
  );
  //return language === "typescript" ? <>{children}</> : null;
};

export const Python: React.FC<LanguageProps> = ({ children }) => {
  const { language } = useLanguage();
  return (
    <div className={cn(language === "python" ? "" : "hidden")}>{children}</div>
  );
  //return language === "python" ? <>{children}</> : null;
};

export const LanguageSwitch: React.FC<{
  typescript: ReactNode;
  python: ReactNode;
}> = ({ typescript, python }) => {
  const { language } = useLanguage();
  return <>{language === "typescript" ? typescript : python}</>;
};
