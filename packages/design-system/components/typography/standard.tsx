import { Copy } from "lucide-react";
import { forwardRef } from "react";
import { ReactNode } from "react";
import { cn } from "../../lib/utils";
import { type ComponentPropsWithoutRef } from "react";

export const BannerDisplay = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <h1
      className={cn(
        "text-primary text-nowrap text-9xl md:text-[20rem] ",
        className
      )}
    >
      {children}
    </h1>
  );
};

const textBase = "my-5";

export const SuperDisplay = ({
  className,
  children,
}: ComponentPropsWithoutRef<"h1">) => {
  return (
    <h1
      className={cn(
        "text-primary text-4xl xs:text-5xl sm:text-7xl md:text-8xl lg:text-9xl 2xl:text-[12rem] 3xl::text-[13rem]",
        className
      )}
    >
      {children}
    </h1>
  );
};

export const Display = ({
  className,
  children,
}: ComponentPropsWithoutRef<"h2">) => {
  return (
    <h2
      className={cn(
        "text-primary text-6xl sm:text-8xl 3xl:text-9xl",
        textBase,
        className
      )}
    >
      {children}
    </h2>
  );
};

export const Heading = ({
  className,
  children,
}: ComponentPropsWithoutRef<"h3">) => {
  return (
    <h3
      className={cn(
        "text-primary text-4xl sm:text-6xl 3xl:text-7xl",
        textBase,
        className
      )}
    >
      {children}
    </h3>
  );
};

export const SmallText = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <p
      className={cn(
        "text-primary text-1xl 2xl:text-2xl 3xl:text-3xl",
        textBase,
        className
      )}
    >
      {children}
    </p>
  );
};

export const TextEmbed = forwardRef<
  HTMLSpanElement,
  ComponentPropsWithoutRef<"span">
>(({ className, children, ...props }, ref) => {
  return (
    <span
      ref={ref}
      className={cn("text-primary text-2xl 2xl:text-3xl", textBase, className)}
      {...props}
    >
      {children}
    </span>
  );
});

export const Text = forwardRef<
  HTMLParagraphElement,
  ComponentPropsWithoutRef<"p">
>(({ className, children, ...props }, ref) => {
  return (
    <p
      ref={ref}
      className={cn(
        "text-primary leading-normal 2xl:leading-normal text-lg sm:text-xl 2xl:text-2xl 3xl:text-3xl",
        textBase,
        className
      )}
      {...props}
    >
      {children}
    </p>
  );
});

export const CodeSnippet = ({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) => {
  return (
    <div
      className={cn(
        "text-primary bg-muted rounded-md py-5 px-6 flex flex-row gap-5 cursor-pointer",
        className
      )}
    >
      <Text className="grow my-0">{children}</Text>
      <div>
        <Copy strokeWidth={3} />
      </div>
    </div>
  );
};
