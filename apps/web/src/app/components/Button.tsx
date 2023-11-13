import * as React from "react";

interface ButtonProps {
  children?: React.ReactNode;
  className?: string;
  href?: string;
}

export const ButtonStyle  = ({children, href}: ButtonProps) => {
  return (
    <a className="flex grow items-center justify-center border border-transparent px-8 py-3  text-center font-medium no-underline bg-action-primary text-black hover:bg-gray-300 sm:inline-block sm:grow-0 md:py-3 md:px-10 md:text-lg md:leading-8" href={href}>
      <div>
        {children}
      </div>
    </a>
  );
};