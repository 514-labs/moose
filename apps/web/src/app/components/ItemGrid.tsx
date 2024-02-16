interface GridProps {
  children: React.ReactNode;
  cols: number;
  rows: number;
  className?: string;
  ref?: React.MutableRefObject<HTMLDivElement>;
  itemPosition?: ItemPosition;
}

type ItemPosition = "center" | "start" | "end";

const baseSizeStyles = (itemPosition?: ItemPosition) => {
  switch (itemPosition) {
    case "center":
      return "grid w-full place-items-center ";
    case "start":
      return "grid w-full place-items-start ";
    case "end":
      return "grid w-full place-items-end ";
    default:
      return "grid w-full place-items-center ";
  }
};

const grid_cols = {
  1: {
    sm: "sm:grid-cols-1",
    md: "md:grid-cols-1",
    lg: "lg:grid-cols-1",
    xl: "3xl:grid-cols-1",
  },
  2: {
    sm: "sm:grid-cols-2",
    md: "md:grid-cols-2",
    lg: "lg:grid-cols-2",
    xl: "3xl:grid-cols-2",
  },
  3: {
    sm: "sm:grid-cols-3",
    md: "md:grid-cols-3",
    lg: "lg:grid-cols-3",
    xl: "3xl:grid-cols-3",
  },
};

const grid_rows = {
  1: {
    sm: "sm:grid-rows-1",
    md: "md:grid-rows-1",
    lg: "lg:grid-rows-1",
    xl: "3xl:grid-rows-1",
  },
  2: {
    sm: "sm:grid-rows-2",
    md: "md:grid-rows-2",
    lg: "lg:grid-rows-2",
    xl: "3xl:grid-rows-2",
  },
  3: {
    sm: "sm:grid-rows-3",
    md: "md:grid-rows-3",
    lg: "lg:grid-rows-3",
    xl: "3xl:grid-rows-3",
  },
  4: {
    sm: "sm:grid-rows-4",
    md: "md:grid-rows-4",
    lg: "lg:grid-rows-4",
    xl: "3xl:grid-rows-4",
  },
  6: {
    sm: "sm:grid-rows-6",
    md: "md:grid-rows-6",
    lg: "lg:grid-rows-6",
    xl: "3xl:grid-rows-6",
  },
  9: {
    sm: "sm:grid-rows-9",
    md: "md:grid-rows-9",
    lg: "lg:grid-rows-9",
    xl: "3xl:grid-rows-9",
  },
};

const getSmallStyles = (_cols: number, _rows: number) => {
  return "";
};

const getMediumStyles = (cols: number, rows: number) => {
  return `${grid_cols[cols]["md"]} ${grid_rows[rows]["md"]}`;
};

const getLargeStyles = (cols: number, rows: number) => {
  return `${grid_cols[cols]["lg"]} ${grid_rows[rows]["lg"]}`;
};

const getXLargeStyles = (cols: number, rows: number) => {
  return `${grid_cols[cols]["xl"]} ${grid_rows[rows]["xl"]}`;
};

const composeSizes = (cols: number, rows: number) => {
  const combo = cols * rows;
  let sizeStyles =
    getSmallStyles(1, combo) +
    " " +
    getMediumStyles(1, combo) +
    " " +
    getLargeStyles(cols, rows) +
    " " +
    getXLargeStyles(cols, rows);
  if (cols === 1) {
    sizeStyles += "space-y-10 lg:space-y-24";
  }

  if (cols > 1) {
    sizeStyles += " gap-10";
  }

  return sizeStyles;
};

const getDefaultStyle = (
  className: string,
  cols: number,
  rows: number,
  itemPosition: ItemPosition
) => {
  if (className) {
    return (
      className +
      " " +
      baseSizeStyles(itemPosition) +
      " " +
      composeSizes(cols, rows)
    );
  } else {
    return baseSizeStyles(itemPosition) + " " + composeSizes(cols, rows);
  }
};

export const ItemGrid = ({
  children,
  cols,
  rows,
  className,
  ref,
  itemPosition,
}: GridProps) => {
  return (
    <div
      className={getDefaultStyle(className, cols, rows, itemPosition)}
      ref={ref}
    >
      {children}
    </div>
  );
};
