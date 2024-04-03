"use client";

import Image from "next/image";
import { useTheme } from "next-themes";

export const TemplateImg = ({
  srcDark,
  srcLight,
  alt,
}: {
  srcDark: string;
  srcLight: string;
  alt: string;
}) => {
  const { theme } = useTheme();
  return (
    <>
      <Image
        priority
        className="hidden dark:block"
        src={srcDark}
        fill
        alt={alt}
        sizes=" (max-width: 768px) 150vw, 25vw"
      />

      <Image
        priority
        className="block dark:hidden"
        src={srcLight}
        fill
        alt={alt}
        sizes=" (max-width: 768px) 150vw, 25vw"
      />
    </>
  );
};
