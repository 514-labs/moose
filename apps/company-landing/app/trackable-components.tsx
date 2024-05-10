"use client";
import { withTrack, TrackingVerb } from "event-capture/withTrack";
import { CTAButton, CTAButtonProps } from "@514labs/design-system/components";

export const TrackCTAButton = withTrack<CTAButtonProps>({
  Component: CTAButton,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});
