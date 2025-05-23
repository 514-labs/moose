import React from "react";
import { cn } from "@/lib/utils";
import { Callout } from "@/components";
import {
  PlusCircle,
  RefreshCw,
  Trash2,
  Wrench,
  Shield,
  AlertTriangle,
  Sparkles,
} from "lucide-react";

type ChangelogCategoryType =
  | "highlights"
  | "added"
  | "changed"
  | "deprecated"
  | "fixed"
  | "security"
  | "breaking-changes";

interface ChangelogCategoryProps {
  type: ChangelogCategoryType;
  title?: string;
  children: React.ReactNode;
}

const categoryConfig = {
  highlights: {
    calloutType: "success" as const,
    icon: <Sparkles className="text-moose-green" />,
    defaultTitle: "Release Highlights",
  },
  added: {
    calloutType: "info" as const,
    icon: <PlusCircle className="text-muted-foreground" />,
    defaultTitle: "Added",
  },
  changed: {
    calloutType: "info" as const,
    icon: <RefreshCw className="text-muted-foreground" />,
    defaultTitle: "Changed",
  },
  deprecated: {
    calloutType: "warning" as const,
    icon: <Trash2 className="text-muted-foreground" />,
    defaultTitle: "Deprecated",
  },
  fixed: {
    calloutType: "info" as const,
    icon: <Wrench className="text-muted-foreground" />,
    defaultTitle: "Fixed",
  },
  security: {
    calloutType: "info" as const,
    icon: <Shield className="text-muted-foreground" />,
    defaultTitle: "Security",
  },
  "breaking-changes": {
    calloutType: "danger" as const,
    icon: <AlertTriangle className="text-destructive" />,
    defaultTitle: "Breaking Changes",
  },
};

export function ChangelogCategory({
  type,
  title,
  children,
}: ChangelogCategoryProps) {
  const config = categoryConfig[type];

  return (
    <Callout
      type={config.calloutType}
      title={title || config.defaultTitle}
      icon={config.icon}
    >
      {children}
    </Callout>
  );
}

// Convenience components for each category
export function ReleaseHighlights({ children }: { children: React.ReactNode }) {
  return <ChangelogCategory type="highlights">{children}</ChangelogCategory>;
}

export function Added({ children }: { children: React.ReactNode }) {
  return <ChangelogCategory type="added">{children}</ChangelogCategory>;
}

export function Changed({ children }: { children: React.ReactNode }) {
  return <ChangelogCategory type="changed">{children}</ChangelogCategory>;
}

export function Deprecated({ children }: { children: React.ReactNode }) {
  return <ChangelogCategory type="deprecated">{children}</ChangelogCategory>;
}

export function Fixed({ children }: { children: React.ReactNode }) {
  return <ChangelogCategory type="fixed">{children}</ChangelogCategory>;
}

export function Security({ children }: { children: React.ReactNode }) {
  return <ChangelogCategory type="security">{children}</ChangelogCategory>;
}

export function BreakingChanges({ children }: { children: React.ReactNode }) {
  return (
    <ChangelogCategory type="breaking-changes">{children}</ChangelogCategory>
  );
}
