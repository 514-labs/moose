import { render } from "@/components";

const rawMeta = {
  dev: {
    title: "Development Mode",
  },
  prod: {
    title: "Production Mode",
  },
};

export default render(rawMeta);
