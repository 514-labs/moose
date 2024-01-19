import Metadata from "next";

export const metadata: Metadata = {
  title: "MooseJS | Build for the modern data stack",
  openGraph: {
    images: "/open-graph/og_igloo_4x.webp",
  },
};

interface Route {
  file_path: string;
  route_path: string;
  table_name: string;
  view_name: string;
}

interface Table {
  database: string;
  dependencies_table: string[];
  engine: string;
  name: string;
  uuid: string;
}

interface ConsoleResponse {
  routes: Route[];
  tables: Table[];
  topics: string[];
}

interface RoutesListProps {
  routes: Route[];
}

interface TablesListProps {
  tables: Table[];
}

interface TopicsListProps {
  topics: string[];
}

async function getData(): Promise<ConsoleResponse> {
  const res = await fetch("http://localhost:4000/console", {
    cache: "no-store",
  });
  // The return value is *not* serialized
  // You can return Date, Map, Set, etc.

  if (!res.ok) {
    // This will activate the closest `error.js` Error Boundary
    throw new Error("Failed to fetch data");
  }

  return res.json();
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
  const data = await getData();

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
