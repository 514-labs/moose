import * as React from "react";

interface ButtonProps {
  children?: React.ReactNode;
  className?: string;
  href?: string;
}

export const Button  = ({children}: ButtonProps) => {
  return (
    <a className="ui-flex sm:ui-grow-0 grow ui-items-center ui-justify-center ui-border ui-border-transparent ui-px-8 ui-py-3 ui-text-base ui-text-center ui-font-medium ui-no-underline ui-bg-action-primary ui-text-black hover:ui-bg-gray-300 md:ui-py-3 md:ui-px-10 md:ui-text-lg md:ui-leading-6" href="https://xbac8793a37.typeform.com/to/KTf5bBbr">
      <div >
        {children}
      </div>
    </a>
  );
};
