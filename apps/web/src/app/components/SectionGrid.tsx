interface GridProps {
    children: React.ReactNode,
    gapStyle: string,
    className?: string,
    ref?: React.MutableRefObject<HTMLDivElement>,
    itemPosition?: ItemPosition 
}

type ItemPosition = "center" | "start" | "end";

const baseSizeStyles = (itemPosition?: ItemPosition) => {
    switch (itemPosition) {
        case "center":
            return "grid w-full place-items-center grid-cols-3 ";
        case "start":
            return "grid w-full place-items-start grid-cols-3 ";
        case "end":
            return "grid w-full place-items-end grid-cols-3 ";
        default:
            return "grid w-full place-items-center grid-cols-3 ";
    }
}

const getSmallStyles =  "sm:grid-cols-12";
const getMediumStyles =  "md:grid-cols-12";
const getLargeStyles =  "lg:grid-cols-12";
const getXLargeStyles =  "xl:grid-cols-12";

const composeSizes = () => {
    const sizeStyles = getSmallStyles + " " + getMediumStyles + " " + getLargeStyles + " " + getXLargeStyles;

    return sizeStyles;
}

const getDefaultStyle = (className: string,  itemPosition: ItemPosition, gapStyle) => {
    
    if (className) {
        return className + " " + baseSizeStyles(itemPosition) + " " + composeSizes() + " " + gapStyle;
    } else {
        return baseSizeStyles(itemPosition) + " " + composeSizes() + " " + gapStyle;
    }
}

export const SectionGrid = ({children, className, ref, gapStyle, itemPosition}: GridProps) => {
    
    return (
        <div className={getDefaultStyle(className, itemPosition, gapStyle)} ref={ref}>
            {children}
        </div>
    )
}