"use client";
import { CodeSnippet as AnimatedCodeSnippet } from "@514labs/design-system/typography/animated";
import { withTrack, TrackingVerb } from "@514labs/event-capture/withTrack";
import { CTAButton, CTAButtonProps } from "./page";

export const TrackableCodeSnippet = withTrack({
  Component: AnimatedCodeSnippet,
  action: TrackingVerb.copy,
  injectProps: (onCopy) => ({ onCopy }),
});

export interface TrackingFields {
  name: string;
  subject: string;
}

export const TrackCtaButton = (props: CTAButtonProps & TrackingFields) =>
  withTrack<CTAButtonProps>({
    Component: CTAButton,
    action: TrackingVerb.clicked,
    injectProps: (onClick) => ({ onClick }),
  })(props);
