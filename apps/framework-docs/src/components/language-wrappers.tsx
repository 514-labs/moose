import React, { ReactNode } from "react";
import { useLanguage } from "./LanguageContext";
import { cn } from "@514labs/design-system-components/utils";

interface LanguageProps {
  children: ReactNode;
  className?: string;
}

export const TypeScript: React.FC<LanguageProps> = ({
  children,
  className,
}) => {
  const { language } = useLanguage();
  return (
    <div className={cn(language === "typescript" ? className : "hidden")}>
      {children}
    </div>
  );
  //return language === "typescript" ? <>{children}</> : null;
};

export const Python: React.FC<LanguageProps> = ({ children, className }) => {
  const { language } = useLanguage();
  return (
    <div className={cn(language === "python" ? className : "hidden")}>
      {children}
    </div>
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
