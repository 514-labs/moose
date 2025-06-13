import { cn } from "@/lib/utils";

interface StaggeredCardProps {
  children: React.ReactNode;
  stagger?: "left" | "right";
}

export function StaggeredCard({
  children,
  stagger = "left",
}: StaggeredCardProps) {
  return (
    <div
      className={cn(
        "grid grid-cols-1 gap-6 items-center",
        stagger === "left" ?
          "md:grid-cols-[40%_60%]"
        : "md:grid-cols-[65%_35%]",
      )}
    >
      {children}
    </div>
  );
}

interface StaggeredContentProps {
  children: React.ReactNode;
  title?: string;
}

export function StaggeredContent({ children, title }: StaggeredContentProps) {
  return (
    <div className="p-4">
      {title && <h3 className="text-xl font-semibold mb-2">{title}</h3>}
      {children}
    </div>
  );
}

interface StaggeredCodeProps {
  children: React.ReactNode;
  language?: string;
}

export function StaggeredCode({
  children,
  language = "ts",
}: StaggeredCodeProps) {
  return (
    <div className="border rounded-lg p-4 bg-muted w-full max-w-3xl overflow-x-auto">
      {children}
    </div>
  );
}
