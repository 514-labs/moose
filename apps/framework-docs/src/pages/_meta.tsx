import { render } from "@/components";
import Display from "@/components/display";

export default render({
  index: {
    display: "hidden",
    theme: {
      breadcrumb: false,
      sidebar: false,
    },
  },
  moose: {
    type: "page",
    title: "Moose Stack",
    href: "/moose",
  },
  aurora: {
    type: "page",
    title: "Aurora",
    href: "/aurora",
  },
  blog: {
    title: "Blog",
    type: "page",
    href: "https://www.fiveonefour.com/blog",
    newWindow: true,
  },
  "area-code": {
    type: "page",
    title: "Area Code",
    href: "https://github.com/514-labs/area-code",
    newWindow: true,
  },
  "release-notes": {
    type: "page",
    title: "Release Notes",
    href: "/release-notes",
  },
  "usage-data": {
    display: "hidden",
  },
  templates: {
    display: "hidden",
  },
});
