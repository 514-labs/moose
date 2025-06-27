import {
  Clock,
  Database,
  Waves,
  HardDriveDownload,
  RectangleEllipsis,
  HardDriveUpload,
  Slack,
  Contact,
  Github,
  Youtube,
  Rocket,
  FolderTree,
  PencilRuler,
  Blocks,
  Table,
  Layers,
  Terminal,
  Library,
  Calendar,
  Linkedin,
  ListX,
  FileJson,
  Monitor,
} from "lucide-react";

const XIcon = ({ className, ...props }: React.SVGProps<SVGSVGElement>) => (
  <svg
    className={className}
    fill="currentColor"
    viewBox="0 0 24 24"
    aria-hidden="true"
    {...props}
  >
    <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
  </svg>
);

const ClickHouseIcon = (props: React.SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 50.6 50.6"
    fill="currentColor"
    {...props}
  >
    <path d="M0.6,0H5c0.3,0,0.6,0.3,0.6,0.6V50c0,0.3-0.3,0.6-0.6,0.6H0.6C0.3,50.6,0,50.4,0,50V0.6C0,0.3,0.3,0,0.6,0z" />
    <path d="M11.8,0h4.4c0.3,0,0.6,0.3,0.6,0.6V50c0,0.3-0.3,0.6-0.6,0.6h-4.4c-0.3,0-0.6-0.3-0.6-0.6V0.6C11.3,0.3,11.5,0,11.8,0z" />
    <path d="M23.1,0h4.4c0.3,0,0.6,0.3,0.6,0.6V50c0,0.3-0.3,0.6-0.6,0.6h-4.4c-0.3,0-0.6-0.3-0.6-0.6V0.6C22.5,0.3,22.8,0,23.1,0z" />
    <path d="M34.3,0h4.4c0.3,0,0.6,0.3,0.6,0.6V50c0,0.3-0.3,0.6-0.6,0.6h-4.4c-0.3,0-0.6-0.3-0.6-0.6V0.6C33.7,0.3,34,0,34.3,0z" />
    <path d="M45.6,19.7H50c0.3,0,0.6,0.3,0.6,0.6v10.1c0,0.3-0.3,0.6-0.6,0.6h-4.4c-0.3,0-0.6-0.3-0.6-0.6V20.3C45,20,45.3,19.7,45.6,19.7z" />
  </svg>
);

// Base paths for different sections
const basePaths = {
  start: "/moose/getting-started",
  build: "/moose/building",
  deploy: "/moose/deploying",
  reference: "/moose/reference",
};

// Icon components mapping
export const Icons = {
  // Getting Started
  quickstart: Rocket,
  projectStructure: FolderTree,
  architecture: PencilRuler,
  fromClickhouse: ClickHouseIcon,
  coreConcepts: Blocks,
  // Building
  dataModels: RectangleEllipsis,
  ingestion: HardDriveDownload,
  streams: Waves,
  olapTables: Table,
  workflows: Clock,
  materializedViews: Layers,
  consumptionApis: HardDriveUpload,
  deadLetterQueues: ListX,
  // Reference
  mooseCli: Terminal,
  mooseLibrary: Library,
  // Social
  calendly: Calendar,
  slack: Slack,
  github: Github,
  twitter: XIcon,
  youtube: Youtube,
  linkedin: Linkedin,
  // Legacy mappings for backward compatibility
  models: RectangleEllipsis,
  db: Database,
  api: HardDriveUpload,
  contact: Contact,

  // Aurora
  cli: Terminal,
  json: FileJson,
  computer: Monitor,
};

// Combined paths with icons and metadata
export const PathConfig = {
  // Getting Started
  quickstart: {
    path: `${basePaths.start}/quickstart`,
    icon: Icons.quickstart,
    title: "Quickstart",
    category: "getting-started" as const,
  },
  projectStructure: {
    path: `${basePaths.start}/project-structure`,
    icon: Icons.projectStructure,
    title: "Project Structure",
    category: "getting-started" as const,
  },
  architecture: {
    path: `${basePaths.start}/architecture`,
    icon: Icons.architecture,
    title: "Architecture",
    category: "getting-started" as const,
  },
  fromClickhouse: {
    path: `${basePaths.start}/from-clickhouse`,
    icon: Icons.fromClickhouse,
    title: "From ClickHouse",
    category: "getting-started" as const,
  },
  coreConcepts: {
    path: `${basePaths.start}/core-concepts`,
    icon: Icons.coreConcepts,
    title: "Core Concepts",
    category: "getting-started" as const,
  },
  // Building
  dataModels: {
    path: `${basePaths.build}/data-modeling`,
    icon: Icons.dataModels,
    title: "Data Modeling",
    category: "building" as const,
  },
  ingestion: {
    path: `${basePaths.build}/ingestion`,
    icon: Icons.ingestion,
    title: "Ingestion",
    category: "building" as const,
  },
  streams: {
    path: `${basePaths.build}/streams`,
    icon: Icons.streams,
    title: "Streams",
    category: "building" as const,
  },
  olapTables: {
    path: `${basePaths.build}/olap-table`,
    icon: Icons.olapTables,
    title: "OLAP Tables",
    category: "building" as const,
  },
  workflows: {
    path: `${basePaths.build}/workflows`,
    icon: Icons.workflows,
    title: "Workflows",
    category: "building" as const,
  },
  materializedViews: {
    path: `${basePaths.build}/materialized-views`,
    icon: Icons.materializedViews,
    title: "Materialized Views",
    category: "building" as const,
  },
  consumptionApis: {
    path: `${basePaths.build}/consumption-apis`,
    icon: Icons.consumptionApis,
    title: "Consumption APIs",
    category: "building" as const,
  },
  deadLetterQueues: {
    path: `${basePaths.build}/dead-letter-queues`,
    icon: Icons.deadLetterQueues,
    title: "Dead Letter Queues",
    category: "building" as const,
  },
  // Reference
  mooseCli: {
    path: `${basePaths.reference}/moose-cli`,
    icon: Icons.mooseCli,
    title: "Moose CLI",
    category: "reference" as const,
  },
  mooseLibrary: {
    path: `${basePaths.reference}/moose-lib`,
    icon: Icons.mooseLibrary,
    title: "Moose Library",
    category: "reference" as const,
  },
  // Social
  calendly: {
    path: "https://cal.com/team/514/talk-to-eng",
    icon: Icons.calendly,
    title: "Schedule a Call",
    category: "social" as const,
  },
  slack: {
    path: "https://join.slack.com/t/moose-community/shared_invite/zt-32tt66s7r-WS9wn~FAFEe31cFBtJeaFA",
    icon: Icons.slack,
    title: "Join Slack",
    category: "social" as const,
  },
  github: {
    path: "https://github.com/514-labs/moose",
    icon: Icons.github,
    title: "GitHub",
    category: "social" as const,
  },
  twitter: {
    path: "https://x.com/514hq",
    icon: Icons.twitter,
    title: "Twitter",
    category: "social" as const,
  },
  youtube: {
    path: "https://www.youtube.com/channel/UCmIj6NoAAP7kOSNYk77u4Zw",
    icon: Icons.youtube,
    title: "YouTube",
    category: "social" as const,
  },
  linkedin: {
    path: "https://www.linkedin.com/company/fiveonefour",
    icon: Icons.linkedin,
    title: "LinkedIn",
    category: "social" as const,
  },
} as const;

// Helper functions for easy access
export const getPathByKey = (key: keyof typeof PathConfig) =>
  PathConfig[key].path;
export const getIconByKey = (key: keyof typeof PathConfig) =>
  PathConfig[key].icon;
export const getTitleByKey = (key: keyof typeof PathConfig) =>
  PathConfig[key].title;
export const getCategoryByKey = (key: keyof typeof PathConfig) =>
  PathConfig[key].category;

// Get all paths by category
export const getPathsByCategory = (
  category: "getting-started" | "building" | "reference" | "social",
) => {
  return Object.entries(PathConfig).filter(
    ([_, config]) => config.category === category,
  );
};

// Get all paths as a simple object (for backward compatibility)
export const paths = Object.fromEntries(
  Object.entries(PathConfig).map(([key, config]) => [key, config.path]),
);

// Type definitions
export type PathKey = keyof typeof PathConfig;
export type PathCategory =
  | "getting-started"
  | "building"
  | "reference"
  | "social";
export type IconName = keyof typeof Icons;
