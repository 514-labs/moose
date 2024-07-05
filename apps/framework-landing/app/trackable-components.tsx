"use client";
import { CodeSnippet as AnimatedCodeSnippet } from "@514labs/design-system-components/typography/animated";
import { withTrack, TrackingVerb } from "@514labs/event-capture/withTrack";
import { CTAButton, CTAButtonProps } from "./page";
import { AccordionTrigger } from "@514labs/design-system-components/components";

export const TrackableCodeSnippet = withTrack({
  Component: AnimatedCodeSnippet,
  action: TrackingVerb.copy,
  injectProps: (onCopy) => ({ onCopy }),
});

export interface TrackingFields {
  name: string;
  subject: string;
  targetUrl?: string;
}

export const TrackCtaButton = (props: CTAButtonProps & TrackingFields) =>
  withTrack<CTAButtonProps>({
    Component: CTAButton,
    action: TrackingVerb.clicked,
    injectProps: (onClick) => ({ onClick }),
  })(props);

export const TrackableAccordionTrigger = withTrack({
  Component: AccordionTrigger,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});
