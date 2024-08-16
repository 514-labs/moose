import { Pyramid } from "lucide-react";
import { cn } from "../lib/utils";
import Link from "next/link";

export const ProductBadge = ({
  name,
  tag,
  tagStyle,
}: {
  name: string;
  tag: string;
  tagStyle?: string;
}) => {
  return (
    <Link href="/" className="text-base flex items-center flex-row">
      <span className="mb-0.5">{name}</span>

      <ProductTag tag={tag} tagStyle={tagStyle} />
    </Link>
  );
};

export const ProductTag = ({
  tag,
  tagStyle,
}: {
  tag: string;
  tagStyle?: string;
}) => {
  const gradients = {
    js: "bg-gradient-to-t from-[#3A36FF]  to-[#00A4C8]",
    py: "bg-gradient-to-br from-[#B800C8] to-[#7F00FF]",
  };

  return (
    <div
      className={cn(
        `ml-1 inline-block px-1 rounded-full border-2 indigo-500`,
        tagStyle,
      )}
    >
      <div className="rounded-lg px-1 py-0.5">
        <span
          className={cn(
            `flex items-center bg-clip-text uppercase text-sm text-transparent text-muted-foreground`,
            tagStyle,
          )}
        >
          {tag}
        </span>
      </div>
    </div>
  );
};
