import { render } from "@/components";
// Raw meta object - more concise without repetitive rendering logic
const rawMeta = {
  index: {
    title: "Release Notes",
    theme: {
      breadcrumb: false,
    },
  },
  upcoming: { display: "hidden" }, // This hides it from sidebar/navigation
};

// Process the raw meta object to generate the final meta object with proper rendering
export default render(rawMeta);
