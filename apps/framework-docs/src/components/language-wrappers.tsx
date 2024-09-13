import React, { ReactNode } from "react";
import { useLanguage } from "./LanguageContext";

interface LanguageProps {
  children: ReactNode;
}

export const TypeScript: React.FC<LanguageProps> = ({ children }) => {
  const { language } = useLanguage();
  return language === "typescript" ? <>{children}</> : null;
};

export const Python: React.FC<LanguageProps> = ({ children }) => {
  const { language } = useLanguage();
  return language === "python" ? <>{children}</> : null;
};

export const LanguageSwitch: React.FC<{
  typescript: ReactNode;
  python: ReactNode;
}> = ({ typescript, python }) => {
  const { language } = useLanguage();
  return <>{language === "typescript" ? typescript : python}</>;
};
