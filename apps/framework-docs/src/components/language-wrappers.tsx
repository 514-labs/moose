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
};

export const TypeScriptInline: React.FC<LanguageProps> = ({ children }) => {
  const { language } = useLanguage();
  return (
    <span className={cn(language === "typescript" ? "inline" : "hidden")}>
      {children}
    </span>
  );
};

export const Python: React.FC<LanguageProps> = ({ children }) => {
  const { language } = useLanguage();
  return (
    <div className={cn(language === "python" ? "" : "hidden")}>{children}</div>
  );
};

export const PythonInline: React.FC<LanguageProps> = ({ children }) => {
  const { language } = useLanguage();
  return (
    <span className={cn(language === "python" ? "inline" : "hidden")}>
      {children}
    </span>
  );
};

export const LanguageSwitch: React.FC<{
  typescript: ReactNode;
  python: ReactNode;
}> = ({ typescript, python }) => {
  const { language } = useLanguage();
  return <>{language === "typescript" ? typescript : python}</>;
};
