"use client";
import React, { useState } from "react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  Button,
} from "@514labs/design-system-components/components";
import { ChevronRight } from "lucide-react";
import { SmallText } from "@514labs/design-system-components/typography";
import { cn } from "@514labs/design-system-components/utils";

interface ToggleBlockProps {
  openText: string;
  closeText: string;
  children: React.ReactNode;
  open?: boolean;
}

export function ToggleBlock({
  openText,
  closeText,
  children,
  open,
}: ToggleBlockProps) {
  const [isOpen, setIsOpen] = useState(open);

  return (
    <Collapsible
      open={isOpen}
      onOpenChange={setIsOpen}
      className="w-full space-y-2"
    >
      <CollapsibleTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="flex flex-row items-center justify-start mb-4 w-full"
        >
          <ChevronRight
            className={cn(
              "mr-2 h-4 w-4 transition-transform duration-200",
              isOpen && "transform rotate-90",
            )}
          />
          <SmallText>{isOpen ? closeText : openText}</SmallText>
        </Button>
      </CollapsibleTrigger>
      <CollapsibleContent className="space-y-2">{children}</CollapsibleContent>
    </Collapsible>
  );
}
