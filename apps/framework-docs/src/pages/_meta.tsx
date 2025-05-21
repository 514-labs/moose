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
    title: "Moose",
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
  templates: {
    type: "page",
    title: "Templates",
    href: "/templates",
  },
  changelog: {
    type: "page",
    title: "Release Notes",
    href: "/release-notes",
  },
  "usage-data": {
    display: "hidden",
  },
});
