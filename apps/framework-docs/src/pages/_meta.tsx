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
  "sign-in": {
    type: "page",
    title: "Sign In",
    href: "https://www.boreal.cloud/sign-in",
    newWindow: true,
    display: "menu",
  },
  "usage-data": {
    display: "hidden",
  },
});
