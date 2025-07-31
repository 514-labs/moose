import { render } from "@/components";

const rawMeta = {
  "---Managing Your Database---": {
    title: "Managing Your Schema",
    type: "separator",
  },
  "create-table": {
    title: "Creating Tables",
  },
  "create-materialized-view": {
    title: "Creating Materialized Views",
  },
  "supported-types": {
    title: "Supported Column Types",
  },
  "schema-optimization": {
    title: "Schema Optimization",
  },
  "schema-migrations": {
    title: "Migrations & Schema Changes",
  },
  "---Interacting with Your Database---": {
    title: "Reading and Writing Data",
    type: "separator",
  },
  "insert-data": {
    title: "Inserting Data",
  },
  "query-data": {
    title: "Querying Data",
  },
};

// Process the raw meta object to generate the final meta object with proper rendering
export default render(rawMeta);
