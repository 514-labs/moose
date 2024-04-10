"use server";
import { headers } from "next/headers";
import Mixpanel from "mixpanel";

function IP() {
  const FALLBACK_IP_ADDRESS = "0.0.0.0";
  const forwardedFor = headers().get("x-forwarded-for");

  if (forwardedFor) {
    return forwardedFor.split(",")[0] ?? FALLBACK_IP_ADDRESS;
  }

  return headers().get("x-real-ip") ?? FALLBACK_IP_ADDRESS;
}

export const sendServerEvent = async (name: string, event: any) => {
  console.log("Send Server Event");
  const headersList = headers();
  const host = headersList.get("host");
  const referrer = headersList.get("referer");
  const ip = IP();

  const env = process.env.NODE_ENV;

  const enhancedEvent = { host, env, referrer, ip, ...event };

  const mixpanel = Mixpanel.init("be8ca317356e20c587297d52f93f3f9e");

  mixpanel.track(name, enhancedEvent);
};
