import React from "react";
import { Section } from "@514labs/design-system-components/components/containers";
import {
  Heading,
  Text,
  HeadingLevel,
} from "@514labs/design-system-components/typography";

interface SectionLayoutProps {
  children: React.ReactNode;
  className?: string;
}

export const SectionLayout: React.FC<SectionLayoutProps> = ({
  children,
  className,
}) => {
  return (
    <Section className={`mx-auto max-w-5xl ${className}`}>{children}</Section>
  );
};

interface SectionHeaderProps {
  title: string;
  subtitle?: string;
}

export const SectionHeader: React.FC<SectionHeaderProps> = ({
  title,
  subtitle,
}) => {
  return (
    <div className="text-center mb-12">
      <Heading level={HeadingLevel.l2} className="mb-4">
        {title}
      </Heading>
      {subtitle && <Text className="text-muted-foreground">{subtitle}</Text>}
    </div>
  );
};
