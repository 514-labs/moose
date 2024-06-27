"use client";
import { Button } from "@514labs/design-system-components/components";
import { useRef, lazy } from "react";

const SplineNext = lazy(() => import("@splinetool/react-spline"));

interface DiagramProps {
  spline: any;
}
export default function Diagram({ spline }: DiagramProps) {
  function onLoad(splineApp: any) {
    // save the app in a ref for later use
    spline.current = splineApp;
  }

  function triggerAnimation() {
    console.log(spline.current);
    console.log(spline.current.emitEvent("mousedown", "BOTTOM"));
  }

  function getEvents() {
    console.log(spline.current.getSplineEvents());
  }

  return (
    <SplineNext
      scene="https://prod.spline.design/wFxf34Wj4ZYM3DfR/scene.splinecode"
      onLoad={onLoad}
    />
  );
}
