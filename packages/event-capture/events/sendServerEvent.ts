"use server";
import { cookies, headers } from "next/headers";
import Mixpanel from "mixpanel";
import { NextRequest } from "next/server";

export type PageViewEventProperties = {
  eventId: string;
  timestamp: Date;
  session_id: string;
  user_agent: string;
  locale: string;
  location: string;
  href: string;
  pathname: string;
  referrer: string;
  ip?: string;
};

export type TrackEvent = {
  name: string;
  action: string;
  subject: string;
  targetUrl?: string;
};

function IP() {
  const FALLBACK_IP_ADDRESS = "0.0.0.0";
  const forwardedFor = headers().get("x-forwarded-for");

  if (forwardedFor) {
    return forwardedFor.split(",")[0] ?? FALLBACK_IP_ADDRESS;
  }

  return headers().get("x-real-ip") ?? FALLBACK_IP_ADDRESS;
}

async function mixpanelAsyncTrack(
  event: string,
  properties: object,
  mixpanel: Mixpanel.Mixpanel,
) {
  return new Promise((resolve, reject) => {
    mixpanel.track(event, properties, (err) => {
      if (err) {
        reject(err);
      } else {
        resolve("success");
      }
    });
  });
}

export const setPageViewEventProperties = (
  req: NextRequest,
): PageViewEventProperties => {
  // if there is a request, get the pathname; otherwise pathname should be passed in
  const eventId = crypto.randomUUID();
  const timestamp = new Date();

  const { pathname, href } = req.nextUrl;

  const referrer = req.headers.get("referer") || "";
  const user_agent = req.headers.get("user-agent") || "";
  const locale = req.headers.get("accept-language") || "";

  const ip = IP();
  const location =
    req.geo?.country + ", " + req.geo?.region + ", " + req.geo?.city;

  const session_id = cookies().get("session-id")?.value || "unknown";

  return {
    eventId,
    timestamp,
    session_id,
    user_agent,
    locale,
    location,
    href,
    pathname,
    referrer,
    ip,
  };
};

export const sendTrackEvent = async (
  pathname: string,
  properties: TrackEvent,
) => {
  const eventId = crypto.randomUUID();
  const timestamp = new Date();

  const headersList = headers();
  const host = headersList.get("host") || "";
  const referrer = headersList.get("referer") || "";
  const session_id = cookies().get("session-id")?.value || "unknown";
  const user_agent = headersList.get("user-agent") || "";
  const locale = headersList.get("accept-language") || "";
  const location = locale.split("_")[1] || "unknown";

  const ip = IP();

  const payload = {
    eventId,
    timestamp,
    session_id,
    user_agent,
    href: host + pathname,
    pathname,
    referrer,
    ip,
    locale,
    location,
    ...properties,
  };

  const env = process.env.NODE_ENV;

  const mooseHost =
    env == "development"
      ? "http://localhost:4000"
      : "https://moosefood.514.dev";

  try {
    await fetch(`${mooseHost}/ingest/TrackEvent`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(payload),
    });
  } catch (error) {
    console.error(error);
  }

  const mixpanel = Mixpanel.init("be8ca317356e20c587297d52f93f3f9e");
  await mixpanelAsyncTrack(
    "TrackEvent",
    { host, env, referrer, ip, ...properties },
    mixpanel,
  );
};

export const sendServerEvent = async (eventName: string, event: any) => {
  const mixpanel = Mixpanel.init("be8ca317356e20c587297d52f93f3f9e");
  const headersList = headers();
  const host = headersList.get("host");
  const referrer = headersList.get("referer");

  const ip = IP();

  const env = process.env.NODE_ENV;

  const mixpanelEvent = { host, env, referrer, ip, ...event };

  await mixpanelAsyncTrack(eventName, mixpanelEvent, mixpanel);
};
