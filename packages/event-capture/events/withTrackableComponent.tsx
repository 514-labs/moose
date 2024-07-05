"use client";
import React from "react";
import { sendTrackEvent } from "./sendServerEvent";

export enum TrackingVerb {
  copy = "copy",
  clicked = "clicked",
  submit = "submit",
  selected = "selected",
  changed = "changed",
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
  injectProps: (event: () => Promise<void>) => {
    [key: string]: () => Promise<void>;
  };
  action: TrackingVerb;
}) {
  const trackableComponent = (
    props: T & {
      children?: React.ReactNode;
      name: string;
      subject: string;
      targetUrl?: string;
    },
  ) => {
    const trackEvent = () =>
      sendTrackEvent(window.location.pathname, {
        name: props.name,
        action,
        subject: props.subject,
        targetUrl: props.targetUrl,
      });

    const newProps = injectProps(trackEvent);

    const combined = combineCallbacks(props, newProps);
    if (
      typeof window !== "undefined" &&
      window.localStorage.getItem("trackView") === "true"
    ) {
      return (
        <div className="w-fit h-fit relative">
          <div className="h-full w-full z-10 bg-blue-200 absolute top-0 left-0 hover:opacity-100 opacity-50 hover:[&>*]:bg-accent hover:[&>*]:opacity-100">
            <div className="text-wrap min-h-full h-fit z-10 min-w-full w-max border-red-100 border-2 text-foreground flex flex-col te w-full text-xs opacity-0">
              <span>
                <span className="font-bold">Event:</span> {props.name}
              </span>
              <span>
                <span className="font-bold">Component:</span> {Component.name}
              </span>
              <span className="text-wrap">
                <span className="font-bold">Subject:</span> {props.subject}
              </span>
              <span>
                <span className="font-bold">Action:</span> {action}
              </span>
            </div>
          </div>
          <Component {...(props as T)} {...combined}>
            {props.children}
          </Component>
        </div>
      );
    }

    return (
      <Component {...(props as T)} {...combined}>
        {props.children}
      </Component>
    );
  };

  return trackableComponent;
}
