import { render } from "@/components";

const meta = {
  "---Building & Packaging---": {
    title: "Preparing for Deployment",
    type: "separator",
  },
  "packaging-moose-for-deployment": {
    title: "Packaging Moose for deployment",
  },
  "preparing-clickhouse-redpanda": {
    title: "Preparing Infrastructure",
  },
  "configuring-moose-for-cloud": {
    title: "Cloud Configuration",
  },
  "---Deployment---": {
    title: "Deployment Guides",
    type: "separator",
  },
  "deploying-on-kubernetes": {
    title: "Kubernetes",
  },
  "deploying-on-ecs": {
    title: "AWS ECS",
  },
  "deploying-on-an-offline-server": {
    title: "Offline",
  },
  "deploying-with-docker-compose": {
    title: "Docker Compose",
  },
  "---Monitoring---": {
    title: "Production Monitoring",
    type: "separator",
  },
  monitoring: {
    title: "Monitoring Your App",
  },
};

export default render(meta);
