"use client";
import { withTrack, TrackingVerb } from "event-capture/withTrack";
import { CTAButton, CTAButtonProps } from "../components.tsx/CTAs";

export const TrackCTAButton = withTrack<CTAButtonProps>({
  Component: CTAButton,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});
