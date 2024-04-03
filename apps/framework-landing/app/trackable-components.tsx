"use client";
import { CodeSnippet as AnimatedCodeSnippet } from "design-system/typography/animated";
import { withTrack, TrackingVerb } from "event-capture/withTrack";
import { CTAButton, CTAButtonProps } from "./page";

export const TrackableCodeSnippet = withTrack({
  Component: AnimatedCodeSnippet,
  action: TrackingVerb.copy,
  injectProps: (onCopy) => ({ onCopy }),
});

export const TrackCtaButton = withTrack<CTAButtonProps>({
  Component: CTAButton,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});
