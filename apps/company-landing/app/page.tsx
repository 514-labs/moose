import { ReactNode } from "react";

import { VariantProps } from "class-variance-authority";
import { EmailSection } from "./sections/EmailSection";
import { FooterSection } from "./sections/FooterSection";
import { cn } from "design-system/utils";

import { Button, buttonVariants } from "design-system/components";

import { Text } from "design-system/typography";
import { ManifestoSection } from "./sections/home/manifesto-section";

export const PlaceholderImage = ({ className }: { className?: string }) => {
  return <div className={cn("relative ", className)}> </div>;
};

export const CTAText = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <div
      className={cn(
        "text-center md:text-start text-primary text-4xl bg-muted rounded-md py-5 px-10 text-nowrap",
        className,
      )}
    >
      {children}
    </div>
  );
};

export interface CTAButtonProps extends VariantProps<typeof buttonVariants> {
  className?: string;
  children: ReactNode;
}

export const CTAButton = ({ className, children, variant }: CTAButtonProps) => {
  return (
    <Button
      size={"lg"}
      variant={variant}
      className="px-6 h-full font-normal border-primary"
    >
      <Text
        className={cn(
          variant === "outline" ? "text-primary" : "text-primary-foreground",
          className,
        )}
      >
        {children}
      </Text>
    </Button>
  );
};

export const CTABar = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <div className={cn("flex flex-col md:flex-row gap-5", className)}>
      {children}
    </div>
  );
};

export default function Home() {
  return (
    <main className="min-h-screen">
      <ManifestoSection />
      <FooterSection />
      <EmailSection />
    </main>
  );
}
