import { headers } from "next/headers";

function IP() {
  const FALLBACK_IP_ADDRESS = "0.0.0.0";
  const forwardedFor = headers().get("x-forwarded-for");

  if (forwardedFor) {
    return forwardedFor.split(",")[0] ?? FALLBACK_IP_ADDRESS;
  }

  return headers().get("x-real-ip") ?? FALLBACK_IP_ADDRESS;
}

export const sendServerEvent = async (name: string, event: any) => {
  const headersList = headers();
  const host = headersList.get("host");
  const referer = headersList.get("referer");
  const ip = IP();

  const env = process.env.NODE_ENV;

  const scheme = env === "production" ? "https" : "http";
  const url = `${scheme}://${host}/events/api`;

  const res = fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      name,
      event: { ...event, host, env, referer, ip },
    }),
  });

  return { ip };
};
