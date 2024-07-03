"use client";
import { lazy } from "react";

const SplineNext = lazy(() => import("@splinetool/react-spline"));

interface DiagramProps {
  spline: any;
  height: number;
}
export default function Diagram({ spline, height }: DiagramProps) {
  function onLoad(splineApp: any) {
    // save the app in a ref for later use
    spline.current = splineApp;
  }
  return (
    <div style={{ height: height }} className="pointer-events-none">
      <SplineNext
        scene="https://prod.spline.design/RJScsp86rZc2qKSB/scene.splinecode"
        onLoad={onLoad}
      />
    </div>
  );
}

/*
<SplineNext
scene="https://prod.spline.design/RJScsp86rZc2qKSB/scene.splinecode"
onLoad={onLoad}
/>
*/
