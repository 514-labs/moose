import React from "react";
import { cn } from "@/lib/utils";
import { SmallTextEmbed } from "@/components/typography";
import { Lightbulb, StopCircle, FileWarning, PartyPopper } from "lucide-react";
import { Card, CardContent } from "@/components/ui";
import Link from "next/link";

interface CalloutProps {
  type: CalloutType;
  title?: string;
  href?: string;
  children: React.ReactNode;
}

const calloutVariants = {
  success: {
    icon: <PartyPopper className="text-moose-green" />,
    color: "bg-muted/20",
    border: "border-moose-green/20",
    title: "Congrats!",
    titleColor: "text-moose-green",
  },
  info: {
    icon: <Lightbulb className="text-muted-foreground" />,
    color: "bg-card",
    border: "border",
    title: "MooseTip:",
    titleColor: "text-muted-foreground",
  },
  warning: {
    icon: <StopCircle className="text-yellow" />,
    color: "bg-muted/20",
    border: "border-moose-yellow/20",
    title: "Warning:",
    titleColor: "text-muted-foreground",
  },
  danger: {
    icon: <FileWarning className="text-descructive" />,
    color: "bg-muted/20",
    border: "border-destructive/20",
    title: "Error:",
    titleColor: "text-muted-foreground",
  },
};

type CalloutType = keyof typeof calloutVariants;

export function Callout({ type, title, href, children }: CalloutProps) {
  const variantProps = calloutVariants[type];

  const TitleContent = () => (
    <p
      className={cn(
        "font-semibold my-0 inline-block mr-2",
        variantProps.titleColor,
        href && "text-moose-purple hover:underline cursor-pointer",
      )}
    >
      {title || variantProps.title}
    </p>
  );

  return (
    <Card
      className={cn(
        "callout border b-[1px] my-5 flex items-start w-full p-4 space-x-2",
        variantProps.color,
        variantProps.border,
      )}
    >
      <CardContent className="flex items-start space-x-2 p-0">
        <div className="flex-shrink-0 mt-1">{variantProps.icon}</div>
        <div className="flex-1">
          {href ? (
            <Link href={href}>
              <TitleContent />
            </Link>
          ) : (
            <TitleContent />
          )}
          {children}
        </div>
      </CardContent>
    </Card>
  );
}
