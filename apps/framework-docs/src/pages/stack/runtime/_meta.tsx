import { render } from "@/components";

const rawMeta = {
  dev: {
    title: "Development Mode",
  },
  prod: {
    title: "Production Mode",
  },
  observability: {
    title: "Observability",
  },
};

export default render(rawMeta);
