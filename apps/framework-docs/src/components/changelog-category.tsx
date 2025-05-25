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
    color: "primary",
  },
  added: {
    calloutType: "info" as const,
    icon: <PlusCircle className="text-primary" />,
    defaultTitle: "Added",
    color: "primary",
  },
  changed: {
    calloutType: "info" as const,
    icon: <RefreshCw className="text-primary" />,
    defaultTitle: "Changed",
    color: "primary",
  },
  deprecated: {
    calloutType: "warning" as const,
    icon: <Trash2 className="text-warning" />,
    defaultTitle: "Deprecated",
    color: "warning",
  },
  fixed: {
    calloutType: "info" as const,
    icon: <Wrench className="text-primary" />,
    defaultTitle: "Fixed",
    color: "primary",
  },
  security: {
    calloutType: "info" as const,
    icon: <Shield className="text-primary" />,
    defaultTitle: "Security",
    color: "primary",
  },
  "breaking-changes": {
    calloutType: "danger" as const,
    icon: <AlertTriangle className="text-destructive" />,
    defaultTitle: "Breaking Changes",
    color: "destructive",
  },
};

interface TitleWithIconProps {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
  className?: string;
  color?: string;
}

export function TitleWithIcon({
  title,
  icon,
  children,
  className,
  color,
}: TitleWithIconProps) {
  return (
    <div
      className={cn(
        `flex items-start w-full my-5 p-4 space-x-2 border-b-2 rounded-md`,
        className,
      )}
    >
      <div className="flex-shrink-0 mt-1">{icon}</div>
      <div className="flex-1 mb-2">
        <p className={`font-semibold my-0 inline-block mr-2 text-${color}`}>
          {title}
        </p>
        {children}
      </div>
    </div>
  );
}

export function ChangelogCategory({
  type,
  title,
  children,
}: ChangelogCategoryProps) {
  const config = categoryConfig[type];

  if (type === "highlights" || type === "breaking-changes") {
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
  return (
    <TitleWithIcon
      title={title || config.defaultTitle}
      icon={config.icon}
      color={config.color}
    >
      {children}
    </TitleWithIcon>
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
