import Metadata from "next";
import { gsap } from "gsap";
import { unstable_noStore as noStore } from "next/cache";
import { getCliData, Route, Table } from "./db";

export const metadata: Metadata = {
  title: "MooseJS | Build for the modern data stack",
  openGraph: {
    images: "/open-graph/og_igloo_4x.webp",
  },
};

interface RoutesListProps {
  routes: Route[];
}

interface TablesListProps {
  tables: Table[];
}

interface TopicsListProps {
  topics: string[];
}

const RoutesList = ({ routes }: RoutesListProps) => (
  <ul>
    {routes.map((route, index) => (
      <li key={index}>{route.route_path}</li>
    ))}
  </ul>
);

const TablesList = ({ tables }) => (
  <ul>
    {tables.map((table, index) => (
      <li key={index}>
        {/* Add a link to the table if it's a view */}
        {table.name.includes("_view") ? (
          <a href={`/tables/${table.name}`}>{table.name}</a>
        ) : (
          table.name
        )}
      </li>
    ))}
  </ul>
);

const TopicsList = ({ topics }) => (
  <ul>
    {topics.map((topic, index) => (
      <li key={index}>{topic}</li>
    ))}
  </ul>
);

export default async function Home(): Promise<JSX.Element> {
  // This is to make sure the environment variables are read at runtime
  // and not during build time
  noStore();
  const data = await getCliData();

  return (
    <>
      {/* <h1 className="text-3xl font-bold">Routes</h1>
      <RoutesList routes={data.routes} />

      <h1 className="text-3xl font-bold">Tables</h1>
      <TablesList tables={data.tables} />

      <h1 className="text-3xl font-bold">Topics</h1>
      <TopicsList topics={data.topics} /> */}
    </>
  );
}
