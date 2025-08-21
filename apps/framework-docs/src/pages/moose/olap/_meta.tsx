import { render } from "@/components";

const rawMeta = {
  "---Schema---": {
    title: "Schema",
    type: "separator",
  },
  "model-table": {
    title: "Modeling Tables",
  },
  "model-materialized-view": {
    title: "Modeling Materialized Views",
  },
  "supported-types": {
    title: "Supported Types",
  },
  "schema-optimization": {
    title: "Schema Optimization",
  },
  "---Migrations---": {
    title: "Migrations",
    type: "separator",
  },
  "apply-migrations": {
    title: "Applying Migrations",
  },
  "planned-migrations": {
    title: "Planned Migrations",
  },
  "external-tables": {
    title: "External Tables",
  },
  "schema-versioning": {
    title: "Schema Versioning",
  },
  "schema-change": {
    title: "Failed Migrations",
  },
  "---Accessing Data---": {
    title: "Accessing Data",
    type: "separator",
  },
  "insert-data": {
    title: "Inserting Data",
  },
  "read-data": {
    title: "Reading Data",
  },
};

// Process the raw meta object to generate the final meta object with proper rendering
export default render(rawMeta);
