export const basePaths = {
  start: "/moose/getting-started",
  build: "/moose/building",
  deploy: "/moose/deploying",
  reference: "/moose/reference",
};

export const paths = {
  quickstart: `${basePaths.start}/quickstart`,
  projectStructure: `${basePaths.start}/project-structure`,
  dataModels: `${basePaths.build}/data-modeling`,
  ingestion: `${basePaths.build}/ingestion`,
  streams: `${basePaths.build}/streams`,
  tables: `${basePaths.build}/olap-tables`,
  materializedViews: `${basePaths.build}/materialized-views`,
  consumptionApis: `${basePaths.build}/consumption-apis`,
  mooseCli: `${basePaths.reference}/moose-cli`,
  mooseLibrary: `${basePaths.reference}/moose-lib`,
};
