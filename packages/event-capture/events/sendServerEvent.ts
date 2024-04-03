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

export type ServerEventResponse = Promise<{
  ip: string;
}>;
export const sendServerEvent = async (name: string, event: any) => {
  const headersList = headers();
  const host = headersList.get("host");
  const referrer = headersList.get("referer");
  const ip = IP();

  const env = process.env.NODE_ENV;

  const enhancedEvent = { ...event, host, env, referrer, ip };

  const mixpanel = Mixpanel.init("be8ca317356e20c587297d52f93f3f9e");

  mixpanel.track(name, enhancedEvent);
};
