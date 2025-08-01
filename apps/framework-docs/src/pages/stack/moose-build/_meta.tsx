import { render } from "@/components";

const meta = {
  "packaging-moose-for-deployment": {
    title: "Packaging for Deployment",
  },
  infrastructure: {
    title: "Infrastructure Setup",
  },
  "---Deployment Platforms---": {
    title: "Deployment Platforms",
    type: "separator",
  },
  "deploying-on-kubernetes": {
    title: "Kubernetes",
  },
  "deploying-on-ecs": {
    title: "AWS ECS",
  },
  "deploying-with-docker-compose": {
    title: "Docker Compose",
  },
  "deploying-on-an-offline-server": {
    title: "Offline Server",
  },
};

export default render(meta);
