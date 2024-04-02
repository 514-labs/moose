"use client";
import { Button, ButtonProps } from "components/ui/button";
import { sendServerEvent } from "event-capture/server-event";
import { withTrack, TrackingVerb } from "event-capture/withTrack";
import { usePathname } from "next/navigation";
import { useEffect } from "react";

export const TrackButton = withTrack<ButtonProps>({
  Component: Button,
  action: TrackingVerb.clicked,
  injectProps: (onClick) => ({ onClick }),
});

export function useTrackPageView() {
  const pathName = usePathname();

  useEffect(() => {
    sendServerEvent("page_view", {
      path: pathName,
    });
  }, []);
}
