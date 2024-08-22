import {
  SmallTextEmbed,
  TextEmbed,
} from "@514labs/design-system-components/typography";
import { Badge } from "@514labs/design-system-components/components";
import Link from "next/link";

interface ChipProps {
  label: string;
  href: string;
}

export function ChipBadge({ label, href }: ChipProps) {
  return (
    <Link href={href}>
      <Badge
        variant="outline"
        className="hover:bg-muted cursor-pointer border-muted-foreground"
      >
        <SmallTextEmbed className="px-1.5 py-1 my-0 font-normal text-muted-foreground">
          {label}
        </SmallTextEmbed>
      </Badge>
    </Link>
  );
}