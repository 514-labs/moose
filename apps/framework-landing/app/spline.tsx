"use client";
import { lazy } from "react";

const SplineNext = lazy(() => import("@splinetool/react-spline"));

interface DiagramProps {
  spline: any;
}
export default function Diagram({ spline }: DiagramProps) {
  function onLoad(splineApp: any) {
    // save the app in a ref for later use
    spline.current = splineApp;
  }
  return (
    <SplineNext
      scene="https://prod.spline.design/r-MsWvh4evJXT1uv/scene.splinecode"
      onLoad={onLoad}
    />
  );
}
