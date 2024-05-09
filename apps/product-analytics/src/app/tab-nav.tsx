"use client";

import { Button } from "@/components/ui/button";
import { CardDescription } from "@/components/ui/card";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { usePathname, useRouter } from "next/navigation";

export default function TabNav() {
  const router = useRouter();
  const pathName = usePathname();
  return (
    <>
      {/* <CardDescription>Analysis Type</CardDescription> */}
      <CardHeader className="px-0 py-0">Analysis Type </CardHeader>
      <div className="flex gap-2">
        <Button
          className={`rounded-xl ${pathName == "/insights" ? "bg-accent" : ""}`}
          variant={"outline"}
          onClick={() => router.push("/insights")}
        >
          Engagement
        </Button>
        <Button
          className={`rounded-xl  ${pathName == "/funnels" ? "bg-accent" : ""}`}
          variant={"outline"}
          onClick={() => router.push("/funnels")}
        >
          Funnels
        </Button>
      </div>
    </>
  );
}
