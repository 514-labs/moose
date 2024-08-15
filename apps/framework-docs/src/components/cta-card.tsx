import React from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Button,
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
      <Card className="hover:bg-secondary md:w-64 w-auto h-48 m-4">
        <CardHeader>
          <Icon className="h-5 w-5" />
        </CardHeader>
        <CardContent>
          <CardTitle>{title}</CardTitle>
          <CardDescription className="mt-2">{description}</CardDescription>
        </CardContent>
      </Card>
    </Link>
  );
}
