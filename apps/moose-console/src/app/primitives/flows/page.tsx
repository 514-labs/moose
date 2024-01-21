import { Button } from "components/ui/button";


export default async function FlowsPage(): Promise<JSX.Element> {
  
    return (
      <section className="p-4 max-h-screen overflow-y-auto">
        <div className="py-20">
            <div className="text-9xl">Flows</div>
            <div className="text-muted-foreground py-5">Flows enable you to process your data as it moves through your MooseJS application</div>
        </div>
        <div>
            <div className="text-6xl flex flex-row pb-4">Coming soon </div>
            <div className="text-muted-foreground">Flows are currently under development. Join our community to share your thoughts or contribute</div>
            <Button className="mt-4">Join community</Button>
            

        </div>
      </section>    
    );
}