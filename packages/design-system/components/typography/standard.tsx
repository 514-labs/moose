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
        className,
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
        className,
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
        className,
      )}
    >
      {children}
    </h2>
  );
};

export enum HeadingLevel {
  l1 = "text-primary text-4xl sm:text-6xl 3xl:text-7xl",
  l2 = "text-primary text-2xl sm:text-4xl 3xl:text-5xl",
  l3 = "text-primary text-xl sm:text-3xl 3xl:text-4xl",
  l4 = "text-primary text-lg sm:text-2xl 3xl:text-3xl",
}

interface HeadingProps extends ComponentPropsWithoutRef<"h3"> {
  level?: HeadingLevel;
  longForm?: boolean; // Is the heading part of a long form text?
}

const longFormHeadingBase = textBase + " mt-10 mb-0";

export const Heading = ({
  className,
  children,
  level = HeadingLevel.l1,
  longForm,
}: HeadingProps) => {
  return (
    <h3
      className={cn(
        level,
        longForm ? longFormHeadingBase : textBase,
        className,
      )}
    >
      {children}
    </h3>
  );
};

const smallBodyBase =
  "text-primary leading-normal 2xl:leading-normal sm:text-base 2xl:text-lg 3xl:text-xl";

export const SmallText = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return <p className={cn(smallBodyBase, textBase, className)}>{children}</p>;
};

export const SmallTextEmbed = ({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) => {
  return (
    <span className={cn(smallBodyBase, textBase, className)}>{children}</span>
  );
};

const bodyBase =
  "text-primary leading-normal 2xl:leading-normal sm:text-lg 2xl:text-xl 3xl:text-2xl";

export const TextEmbed = forwardRef<
  HTMLSpanElement,
  ComponentPropsWithoutRef<"span">
>(({ className, children, ...props }, ref) => {
  // This component is used to embed text in another paragraph component

  return (
    <span ref={ref} className={cn(bodyBase, textBase, className)} {...props}>
      {children}
    </span>
  );
});

interface TextProps extends ComponentPropsWithoutRef<"p"> {
  longForm?: boolean; // Is the text part of a long form text?
}

const longFormTextBase = textBase + " my-5";

export const Text = forwardRef<HTMLParagraphElement, TextProps>(
  ({ className, children, longForm, ...props }, ref) => {
    return (
      <p
        ref={ref}
        className={cn(
          bodyBase,
          longForm ? longFormTextBase : textBase,
          className,
        )}
        {...props}
      >
        {children}
      </p>
    );
  },
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
        className,
      )}
    >
      <Text className="grow my-0">{children}</Text>
      <div>
        <Copy strokeWidth={3} />
      </div>
    </div>
  );
};
