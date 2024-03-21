"use client"

import { Button } from "@/components/ui/button"
import { CardDescription } from "@/components/ui/card"
import { usePathname, useRouter } from "next/navigation";


export default function TabNav() {
    const router = useRouter()
    const pathName = usePathname()
    return (<>
        <CardDescription>Analysis Type</CardDescription>
        <div className="flex gap-2">
            <Button className={`rounded-xl ${pathName == "/insights" ? 'bg-accent' : ''}`} variant={"outline"} onClick={() => router.push('/insights')}>Engagement</Button>
            <Button className={`rounded-xl  ${pathName == "/funnels" ? 'bg-accent' : ''}`} variant={"outline"} onClick={() => router.push('/funnels')}>Funnels</Button>
        </div>
    </>)
}