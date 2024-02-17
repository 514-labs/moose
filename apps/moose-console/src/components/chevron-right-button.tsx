import { ChevronRight } from "lucide-react";

import { Button } from "components/ui/button";
import Link from "next/link";

interface ChevronRightButtonProps {
  href: string;
}

export function ChevronRightButton({ href }: ChevronRightButtonProps) {
  return (
    <Link href={href}>
      <Button variant="outline" size="icon">
        <ChevronRight className="h-4 w-4" />
      </Button>
    </Link>
  );
}
