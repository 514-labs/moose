import { render } from "@/components";

const meta = {
  summary: "Summary",
  "packaging-moose-for-deployment": "Packaging Moose for deployment",
  "preparing-clickhouse-redpanda": "Preparing Infrastructure",
  "configuring-moose-for-cloud": "Cloud Configuration",
  "deploying-on-kubernetes": "Kubernetes Deployment",
  "deploying-on-ecs": "AWS ECS Deployment",
  "deploying-on-an-offline-server": "Offline Deployment",
  monitoring: "Monitoring Your App",
} as const;

export default render(meta);
