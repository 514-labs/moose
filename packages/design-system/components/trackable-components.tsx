"use client";
import React from "react";
import { withTrack, TrackingVerb } from "event-capture/withTrack";
import Link from "next/link";

export { Nav } from "../components/base-nav";

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
