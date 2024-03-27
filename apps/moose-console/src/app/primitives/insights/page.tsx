import { NavBreadCrumb } from "components/nav-breadcrumb";
import { Button } from "components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "components/ui/card";
import Link from "next/link";

export default async function InsightsPage(): Promise<JSX.Element> {
  return (
    <section className="p-4 max-h-screen overflow-y-auto">
      <NavBreadCrumb />
      <div className="py-10">
        <div className="text-8xl">Insights</div>
      </div>
      <div className="py-4">Insights help you derive value from your data.</div>
      <div className="grid grid-cols-2 gap-4">
        <div className="col-span-3 xl:col-span-1">
          <Card className="h-full">
            <CardHeader>
              <CardTitle>Coming Soon</CardTitle>
            </CardHeader>
            <CardContent>
              <div>
                Insights are currently under development. Join our community to
                contribute or share your thoughts.
              </div>
            </CardContent>
            <CardFooter>
              <Link href="https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg">
                <Button variant="outline">Join Community</Button>
              </Link>
            </CardFooter>
          </Card>
        </div>
        <div className="col-span-3 xl:col-span-1">
          <Card className="h-full">
            <CardHeader>
              <CardTitle>Why use insights?</CardTitle>
            </CardHeader>
            <CardContent>
              <div>
                Easily turn your data into standardized metrics, dashboards, or
                models.
              </div>
            </CardContent>
            <CardFooter>
              <Link href="https://docs.moosejs.com">
                <Button variant="outline">Visit Docs</Button>
              </Link>
            </CardFooter>
          </Card>
        </div>
      </div>
    </section>
  );
}
