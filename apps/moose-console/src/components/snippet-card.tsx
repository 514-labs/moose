import { Button } from "./ui/button";
import { Card } from "./ui/card";

interface SnippetCardProps {
    title: string;
    children: React.ReactNode;
}

export default function SnippetCard({ title, children }: SnippetCardProps) {
    return (
        <div className="py-4">
            <h2 className="py-2 flex flex-row items-center">
            <span>{title}</span>
            <span className="grow" />
            <Button variant="outline">copy</Button>
            </h2>
            <Card className="rounded-2xl bg-muted p-4 overflow-x-auto flex flex-col">
            {children}
            </Card>
        </div>
    )
}