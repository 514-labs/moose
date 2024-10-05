"use client";
import React from "react";
import { withTrack, TrackingVerb } from "@514labs/event-capture/withTrack";
import Link from "next/link";
import { Button, ButtonProps } from "./ui/button";
import { AccordionTrigger } from "./ui/accordion";
import { TabsTrigger as TabsTrigger } from "./ui/tabs";
import { CodeSnippet as AnimatedCodeSnippet } from "@514labs/design-system-components/typography/animated";

export { Nav } from "../components/base-nav";

export interface TrackingFields {
  name: string;
  subject: string;
  targetUrl?: string;
}

type PropsWithoutRef<P> = P extends any
  ? "ref" extends keyof P
    ? Pick<P, Exclude<keyof P, "ref">>
    : P
  : P;

export type ComponentPropsWithoutRef<T extends React.ElementType> =
  PropsWithoutRef<React.ComponentProps<T>>;

type PrimitiveLinkProps = ComponentPropsWithoutRef<typeof Link> & {
  children: React.ReactNode;
};

export const TrackLink = withTrack<PrimitiveLinkProps>({
  Component: Link,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});

export const TrackableCodeSnippet = withTrack({
  Component: AnimatedCodeSnippet,
  action: TrackingVerb.copy,
  injectProps: (onCopy) => ({ onCopy }),
});

export const TrackButton = (props: ButtonProps & TrackingFields) =>
  withTrack<ButtonProps>({
    Component: Button,
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
