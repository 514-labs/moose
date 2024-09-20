import React, { ReactNode } from "react";
import { useLanguage } from "./LanguageContext";
import { cn } from "@514labs/design-system-components/utils";

interface LanguageProps {
  children: ReactNode;
  inline?: boolean;
}

export const TypeScript: React.FC<LanguageProps> = ({ children, inline }) => {
  const { language } = useLanguage();
  return (
    <div
      className={cn(
        language === "typescript" ? (inline ? "inline" : "") : "hidden",
      )}
    >
      {children}
    </div>
  );
};

export const Python: React.FC<LanguageProps> = ({ children, inline }) => {
  const { language } = useLanguage();
  return (
    <div
      className={cn(
        language === "python" ? (inline ? "inline" : "") : "hidden",
      )}
    >
      {children}
    </div>
  );
};

export const LanguageSwitch: React.FC<{
  typescript: ReactNode;
  python: ReactNode;
}> = ({ typescript, python }) => {
  const { language } = useLanguage();
  return <>{language === "typescript" ? typescript : python}</>;
};
