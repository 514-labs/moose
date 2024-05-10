"use client";
import { Button, ButtonProps } from "components/ui/button";
import { withTrack, TrackingVerb } from "@514labs/event-capture/withTrack";
import { NavigationMenuLink } from "./ui/navigation-menu";
import { NavigationMenuLinkProps } from "@radix-ui/react-navigation-menu";

export const TrackButton = withTrack<ButtonProps>({
  Component: Button,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});

export const NavigationMenuLinkTrack = withTrack<NavigationMenuLinkProps>({
  Component: NavigationMenuLink,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});
