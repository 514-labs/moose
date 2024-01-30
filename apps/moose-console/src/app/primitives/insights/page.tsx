import { Button } from "components/ui/button";


export default async function InsightsPage(): Promise<JSX.Element> {
    return (
      <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-20">
            <div className="text-9xl">Insights</div>
            <div className="text-muted-foreground py-5">Insights help you derive value from your data.</div>
            
            <ul className="list-disc list-inside text-muted-foreground mt-4">
                Easily turn your data into:
                <li>Standardized metric</li>
                <li>Engaging Dashboards</li>
                <li>Predictive models</li>
            </ul>
        </div>
        <div>
            <div className="text-6xl flex flex-row pb-4">Coming soon </div>
            <div className="text-muted-foreground">Insights are currently under development. Join our community to share your thoughts or contribute</div>
            <Button className="mt-4">Join community</Button>
        </div>
      </section>    
    );
}