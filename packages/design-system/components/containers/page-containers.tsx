// Containers are used to layout content across all 12 columns.

import { cn } from "../../lib/utils";

// They are used to contain content and apply padding as the screen resizes
export const FullWidthContentContainer = ({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) => {
  return <div className={cn("col-span-12", className)}>{children}</div>;
};

// Containers are used to layout content across all 6 columns.
// They are used to contain content and apply padding as the screen resizes
export const HalfWidthContentContainer = ({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) => {
  return (
    <div className={cn("col-span-12 md:col-span-6", className)}>{children}</div>
  );
};

// Container for 3 columns (a ~ quarter of the screen). These must often be paired with the order
// property to ensure they are laid out correctly
export const QuarterWidthContentContainer = ({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) => {
  return (
    <div className={cn("col-span-6 md:col-span-3", className)}>{children}</div>
  );
};

// Container for 4 columns (a ~ third of the screen). These must often be paired with the order
// property to ensure they are laid out correctly
export const ThirdWidthContentContainer = ({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) => {
  return (
    <div
      className={cn(
        "col-span-12 md:col-span-6 xl:col-span-4 2xl:col-span-4",
        className,
      )}
    >
      {children}
    </div>
  );
};

// Sections are full width and stretch in the vertical direction to accomodate their children
// Sections govern the gutter for the pages
export const Section = ({
  children,
  className,
  gutterless,
}: {
  children: React.ReactNode;
  className?: string;
  gutterless?: boolean;
}) => {
  return (
    <section
      className={cn(
        "my-24 lg:my-24 2xl:my-36",
        gutterless ? "overflow-hidden" : "px-5",
        className,
      )}
    >
      {children}
    </section>
  );
};

// Grids are used to layout content in a 12 column grid
export const Grid = ({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) => {
  return (
    <div className={cn("grid grid-cols-12 gap-y-5 md:gap-10", className)}>
      {children}
    </div>
  );
};
