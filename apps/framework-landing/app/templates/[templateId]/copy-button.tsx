"use client";
import { TrackCtaButton } from "../../trackable-components";
import { CopyIcon } from "lucide-react";
interface Props {
  children: React.ReactNode;
  subject: string;
  name: string;
  copyText: string;
}
export function CopyButton({ children, subject, name, copyText }: Props) {
  return (
    <TrackCtaButton
      name={name}
      className="flex items-center gap-4"
      subject={subject}
      onClick={() => {
        navigator.clipboard.writeText(copyText);
      }}
    >
      {children}
      <CopyIcon size={24} />
    </TrackCtaButton>
  );
}
