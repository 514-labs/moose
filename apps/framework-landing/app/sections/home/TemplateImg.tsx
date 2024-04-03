"use client";

import Image from "next/image";
import { useTheme } from "next-themes";

export const TemplateImg = ({ srcDark, srcLight, alt }) => {
  const { theme } = useTheme();
  return (
    <>
      {theme && (
        <Image
          priority
          src={theme && theme === "dark" ? srcDark : srcLight}
          fill
          alt={alt}
          sizes=" (max-width: 768px) 150vw, 25vw"
        />
      )}
    </>
  );
};
