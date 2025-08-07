import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import dynamic from "next/dynamic";

// Dynamically import the MDX content
const APISpecification = dynamic(
  () => import("@/content/specifications/api-connector.mdx"),
);

function BlobStorageSpecification() {
  return (
    <div className="prose dark:prose-invert prose-neutral">
      <h2>Blob Storage Connectors</h2>
      <p>
        Connectors for cloud storage services like S3, Azure Blob Storage, and
        Google Cloud Storage.
      </p>
    </div>
  );
}

function DatabaseSpecification() {
  return (
    <div className="prose dark:prose-invert prose-neutral">
      <h2>Database Connectors</h2>
      <p>
        Connectors for various database systems including SQL and NoSQL
        databases.
      </p>
    </div>
  );
}

function SaaSSpecification() {
  return (
    <div className="prose dark:prose-invert prose-neutral">
      <h2>SaaS Connectors</h2>
      <p>
        Connectors for Software-as-a-Service platforms and third-party services.
      </p>
    </div>
  );
}

export default function Specification() {
  return (
    <div className="flex flex-col gap-4 max-w-4xl mx-auto">
      <div className="prose dark:prose-invert prose-neutral">
        <h1 className="text-4xl">Specification</h1>
        <p>
          We provide extensive specifications for each type of connector which
          you can feed into the LLM of your choice to get a connector built for
          you.
        </p>
      </div>

      <Tabs defaultValue="apis" className="gap-8 mt-4">
        <TabsList>
          <TabsTrigger value="apis">APIs</TabsTrigger>
          <TabsTrigger value="blob-storage">Blob Storage</TabsTrigger>
          <TabsTrigger value="databases">Databases</TabsTrigger>
          <TabsTrigger value="saas">SaaS</TabsTrigger>
        </TabsList>

        <TabsContent value="apis">
          <div className="prose dark:prose-invert prose-neutral max-w-none">
            <APISpecification />
          </div>
        </TabsContent>

        <TabsContent value="blob-storage">
          <div className="prose dark:prose-invert prose-neutral">
            <BlobStorageSpecification />
          </div>
        </TabsContent>

        <TabsContent value="databases">
          <div className="prose dark:prose-invert prose-neutral">
            <DatabaseSpecification />
          </div>
        </TabsContent>

        <TabsContent value="saas">
          <div className="prose dark:prose-invert prose-neutral">
            <SaaSSpecification />
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
