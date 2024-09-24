import React, { ReactNode } from "react";
import { useLanguage } from "./LanguageContext";
import { cn } from "@514labs/design-system-components/utils";

interface LanguageProps {
  children: ReactNode;
  NodeType?: "div" | "span";
}

export const TypeScript: React.FC<LanguageProps> = ({
  children,
  NodeType = "div",
}) => {
  const { language } = useLanguage();
  return (
    <NodeType className={cn(language === "typescript" ? "" : "hidden")}>
      {children}
    </NodeType>
  );
  //return language === "typescript" ? <>{children}</> : null;
};

export const Python: React.FC<LanguageProps> = ({
  children,
  NodeType = "div",
}) => {
  const { language } = useLanguage();
  return (
    <NodeType className={cn(language === "python" ? "" : "hidden")}>
      {children}
    </NodeType>
  );
};

export const LanguageSwitch: React.FC<{
  typescript: ReactNode;
  python: ReactNode;
}> = ({ typescript, python }) => {
  const { language } = useLanguage();
  return <>{language === "typescript" ? typescript : python}</>;
};
