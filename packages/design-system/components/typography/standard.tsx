import { cn } from "@/lib/utils";
import { Copy } from "lucide-react";
import { forwardRef } from "react";
import { ReactNode } from "react";

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
        "text-primary text-9xl md:text-[20rem] text-nowrap",
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
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <h1
      className={cn(
        "text-primary text-5xl sm:text-7xl md:text-8xl lg:text-9xl 2xl:text-[12rem]",
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
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <h2
      className={cn("text-primary text-7xl sm:text-9xl", textBase, className)}
    >
      {children}
    </h2>
  );
};

export const Heading = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <h3
      className={cn("text-primary text-5xl sm:text-7xl", textBase, className)}
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
      className={cn("text-primary text-2xl 2xl:text-3xl ", textBase, className)}
    >
      {children}
    </p>
  );
};

interface TextProps extends React.HTMLProps<HTMLParagraphElement> {
  className?: string;
  children: ReactNode;
}

export const Text = forwardRef<HTMLParagraphElement, TextProps>(
  ({ className, children, ...props }, ref) => {
    return (
      <p
        ref={ref}
        className={cn(
          "text-primary text-2xl leading-normal 2xl:text-3xl 2xl:leading-normal",
          textBase,
          className
        )}
        {...props}
      >
        {children}
      </p>
    );
  }
);

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
