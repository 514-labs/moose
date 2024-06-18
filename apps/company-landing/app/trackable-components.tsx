"use client";
import { withTrack, TrackingVerb } from "@514labs/event-capture/withTrack";
import {
  CTAButton,
  CTAButtonProps,
} from "@514labs/design-system-components/components";

export const TrackCTAButton = withTrack<CTAButtonProps>({
  Component: CTAButton,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});
