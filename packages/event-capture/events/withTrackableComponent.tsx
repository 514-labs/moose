"use client";
import { ServerEventResponse, sendServerEvent } from "./sendServerEvent";

/*
interface CtaTrackProps {
  onCopy: () => void;
}
*/
export enum TrackingVerb {
  copy = "copy",
  clicked = "clicked",
  submit = "submit",
  selected = "selected",
}

function combineCallbacks(oldProps: any, newProps: any) {
  return Object.keys(newProps).reduce((acc, key) => {
    // If the same key exists in props and both are functions, create a new function
    if (
      typeof newProps[key] === "function" &&
      typeof oldProps[key] === "function"
    ) {
      acc[key] = (...args: any[]) => {
        oldProps[key](...args);
        newProps[key](...args);
      };
    } else {
      acc[key] = newProps[key];
    }
    return acc;
  }, {} as any);
}

export function withTrack<T>({
  Component,
  injectProps,
  action,
}: {
  Component: React.ComponentType<T>;
  injectProps: (event: () => ServerEventResponse) => {
    [key: string]: () => ServerEventResponse;
  };
  action: TrackingVerb;
}) {
  const trackableComponent = (
    props: T & { children?: React.ReactNode; name: string; subject: string }
  ) => {
    const trackEvent = () =>
      sendServerEvent(props.name, { action, subject: props.subject } as any);

    const newProps = injectProps(trackEvent);

    const combined = combineCallbacks(props, newProps);

    return (
      <Component {...(props as T)} {...combined}>
        {props.children}
      </Component>
    );
  };

  return trackableComponent;
}
