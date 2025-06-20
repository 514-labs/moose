export const basePaths = {
  start: "/moose/getting-started",
  build: "/moose/building",
  deploy: "/moose/deploying",
  reference: "/moose/reference",
};

export const paths = {
  // Getting Started
  quickstart: `${basePaths.start}/quickstart`,
  projectStructure: `${basePaths.start}/project-structure`,
  architecture: `${basePaths.start}/architecture`,
  fromClickhouse: `${basePaths.start}/from-clickhouse`,
  coreConcepts: `${basePaths.start}/core-concepts`,
  // Building
  dataModels: `${basePaths.build}/data-modeling`,
  ingestion: `${basePaths.build}/ingestion`,
  streams: `${basePaths.build}/streams`,
  olapTables: `${basePaths.build}/olap-table`,
  workflows: `${basePaths.build}/workflows`,
  materializedViews: `${basePaths.build}/materialized-views`,
  consumptionApis: `${basePaths.build}/consumption-apis`,
  // Reference
  mooseCli: `${basePaths.reference}/moose-cli`,
  mooseLibrary: `${basePaths.reference}/moose-lib`,
  // Social
  calendly: "https://cal.com/team/514/talk-to-eng",
  slack:
    "https://join.slack.com/t/moose-community/shared_invite/zt-32tt66s7r-WS9wn~FAFEe31cFBtJeaFA",
  github: "https://github.com/514-labs/moose",
  twitter: "https://x.com/514hq",
  youtube: "https://www.youtube.com/channel/UCmIj6NoAAP7kOSNYk77u4Zw",
  linkedin: "https://www.linkedin.com/company/fiveonefour",
};
