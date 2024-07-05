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
    <div style={{ height: "100%" }} className="relative">
      <div
        className="pointer-events-none"
        style={{ position: "absolute", height: 1000 }}
      >
        <SplineNext
          scene="https://prod.spline.design/wZ5u1Z2dz6HEwgz0/scene.splinecode"
          onLoad={onLoad}
        />
      </div>
    </div>
  );
}
