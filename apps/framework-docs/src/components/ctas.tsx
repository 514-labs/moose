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
  Hammer,
  GanttChart,
  GitCompareArrows,
  Workflow,
  Columns3,
  Rss,
  Code,
  HelpCircle,
  ChartBar,
  Settings,
  BookOpen,
  HandMetal,
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
  // Index
  overview: GanttChart,
  gettingStarted: HandMetal,
  quickstart: Rocket,
  addToClickhouse: ClickHouseIcon,
  areaCode: Github,
  projectStructure: FolderTree,
  architecture: PencilRuler,
  fromClickhouse: ClickHouseIcon,
  coreConcepts: Blocks,

  // Fundamental Concepts
  runtime: Rocket,
  dataModels: Columns3,

  // Modules
  olap: Table,
  streaming: Rss,
  workflows: Workflow,
  apis: Code,

  // Tools
  build: Hammer,
  plan: GitCompareArrows,
  metrics: ChartBar,

  // Reference
  apiReference: BookOpen,
  mooseCli: Terminal,
  configuration: Settings,
  help: HelpCircle,

  // Social
  calendly: Calendar,
  slack: Slack,
  github: Github,
  twitter: XIcon,
  youtube: Youtube,
  linkedin: Linkedin,

  // -------LEGACY MAPPINGS-------
  // Building
  ingestion: HardDriveDownload,
  streams: Waves,
  olapTables: Table,
  materializedViews: Layers,
  consumptionApis: HardDriveUpload,
  deadLetterQueues: ListX,
  mooseLibrary: Library,

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

const paths = {
  gettingStarted: "/stack/getting-started",
  runtime: "/stack/runtime",
  dataModels: "/stack/data-modeling",
  olap: "/stack/olap",
  streaming: "/stack/streaming",
  workflows: "/stack/workflows",
  apis: "/stack/apis",
  build: "/stack/moose-build",
  plan: "/stack/plan",
  metrics: "/stack/metrics",
  openApi: "/stack/open-api",
  apiReference: "/stack/api-reference",
  mooseCli: "/stack/moose-cli",
  configuration: "/stack/configuration",
  help: "/stack/help",
};

// Combined paths with icons and metadata
export const PathConfig = {
  // GETTING STARTED
  quickstart: {
    path: `${paths.gettingStarted}/quickstart`,
    icon: Icons.quickstart,
    title: "Quickstart",
    category: "getting-started" as const,
  },
  projectStructure: {
    path: `${paths.gettingStarted}/project-structure`,
    icon: Icons.projectStructure,
    title: "Project Structure",
    category: "getting-started" as const,
  },
  fromClickhouse: {
    path: `${paths.gettingStarted}/from-clickhouse`,
    icon: Icons.fromClickhouse,
    title: "From ClickHouse",
    category: "getting-started" as const,
  },
  // FUNDAMENTALS
  runtime: {
    path: `${paths.runtime}`,
    icon: Icons.runtime,
    title: "Runtime",
    category: "getting-started" as const,
  },
  dataModels: {
    path: `${paths.dataModels}`,
    icon: Icons.dataModels,
    title: "Data Models",
    category: "getting-started" as const,
  },
  // MODULES
  olap: {
    path: `${paths.olap}`,
    icon: Icons.olap,
    title: "OLAP",
    category: "modules" as const,
  },
  streaming: {
    path: `${paths.streaming}`,
    icon: Icons.streaming,
    title: "Streaming",
    category: "modules" as const,
  },
  workflows: {
    path: `${paths.workflows}`,
    icon: Icons.workflows,
    title: "Workflows",
    category: "modules" as const,
  },
  apis: {
    path: `${paths.apis}`,
    icon: Icons.apis,
    title: "APIs",
    category: "modules" as const,
  },
  // TOOLS
  build: {
    path: `${paths.build}`,
    icon: Icons.build,
    title: "Build",
    category: "tools" as const,
  },
  plan: {
    path: `${paths.plan}`,
    icon: Icons.plan,
    title: "Plan",
    category: "tools" as const,
  },
  metrics: {
    path: `${paths.metrics}`,
    icon: Icons.metrics,
    title: "Metrics",
    category: "tools" as const,
  },
  // CLIENT INTEGRATIONS
  openApi: {
    path: `${paths.openApi}`,
    icon: Code,
    title: "OpenAPI",
    category: "client-integrations" as const,
  },
  // REFERENCE
  apiReference: {
    path: `${paths.apiReference}`,
    icon: Icons.apiReference,
    title: "API Reference",
    category: "reference" as const,
  },
  mooseCli: {
    path: `${paths.mooseCli}`,
    icon: Icons.mooseCli,
    title: "Moose CLI",
    category: "reference" as const,
  },
  configuration: {
    path: `${paths.configuration}`,
    icon: Icons.configuration,
    title: "Configuration",
    category: "reference" as const,
  },
  help: {
    path: `${paths.help}`,
    icon: Icons.help,
    title: "Help",
    category: "reference" as const,
  },
  // SOCIAL
  calendly: {
    path: "https://cal.com/team/514/talk-to-eng",
    icon: Icons.calendly,
    title: "Schedule a Call",
    category: "social" as const,
  },
  slack: {
    path: "https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg",
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
  // LEGACY PATHS (PRE_MODULARIZATION)
  architecture: {
    path: `${basePaths.start}/architecture`,
    icon: Icons.architecture,
    title: "Architecture",
    category: "getting-started" as const,
  },
  coreConcepts: {
    path: `${basePaths.start}/core-concepts`,
    icon: Icons.coreConcepts,
    title: "Core Concepts",
    category: "getting-started" as const,
  },
  // BUILDING
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
  mooseLibrary: {
    path: `${basePaths.reference}/moose-lib`,
    icon: Icons.mooseLibrary,
    title: "Moose Library",
    category: "reference" as const,
  },
} as const;
