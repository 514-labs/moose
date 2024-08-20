import React from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@514labs/design-system-components/components";
import Link from "next/link";
import { GraduationCap, Zap } from "lucide-react";

interface CTACardProps {
  title: string;
  description: string;
  ctaLink: string;
  Icon: React.ElementType;
}

export default function CTACard({
  title,
  description,
  ctaLink,
  Icon,
}: CTACardProps) {
  return (
    <Link href={ctaLink}>
      <Card className="hover:bg-secondary md:w-64 w-auto h-auto m-4">
        <CardHeader>
          <div className="bg-muted rounded-md w-fit">
            <Icon className="m-2 h-[20px] w-[20px]" />
          </div>
        </CardHeader>
        <CardContent>
          <CardTitle>{title}</CardTitle>
          <CardDescription className="mt-2">{description}</CardDescription>
        </CardContent>
      </Card>
    </Link>
  );
}
