"use client";
import { CodeSnippet as AnimatedCodeSnippet } from "@514labs/design-system-components/typography/animated";
import { withTrack, TrackingVerb } from "@514labs/event-capture/withTrack";
import { CTAButton, CTAButtonProps } from "./page";
import {
  AccordionTrigger,
  TabsTrigger,
} from "@514labs/design-system-components/components";
import Link from "next/link";

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

export const TrackLink = withTrack({
  Component: Link,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});

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

export const TrackableTabsTrigger = withTrack({
  Component: TabsTrigger,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});
