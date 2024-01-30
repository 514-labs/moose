import { ArrowUpRight } from "lucide-react"
 
import { Button } from "components/ui/button"
import Link from "next/link"

interface LinkOutButtonProps {
    children: React.ReactNode
    href: string
}
 
export function LinkOutButton({children, href}: LinkOutButtonProps) {
  return (
    <Link href={href}>
        <Button>
        {children} <ArrowUpRight className="mr-2 h-4 w-4" />
        </Button>
    </Link>
  )
}