"use client";
import { TrackButton } from "@514labs/design-system-components/trackable-components";
import { cn } from "@514labs/design-system-components/utils";

interface Props {
  children: React.ReactNode;
  subject: string;
  name: string;
  copyText: string;
  className?: string;
}
export function CopyButton({
  children,
  subject,
  name,
  copyText,
  className,
}: Props) {
  return (
    <TrackButton
      variant="outline"
      name={name}
      className={cn("flex items-center gap-4", className)}
      subject={subject}
      onClick={() => {
        navigator.clipboard.writeText(copyText);
      }}
    >
      {children}
      {/* <CopyIcon size={24} /> */}
    </TrackButton>
  );
}
