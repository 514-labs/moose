import { headers } from "next/headers";

export const sendServerEvent = async (name: string, event: any) => {
  const headersList = headers();
  const host = headersList.get("host");
  const referer = headersList.get("referer");

  const env = process.env.NODE_ENV;

  const scheme = env === "production" ? "https" : "http";
  const url = `${scheme}://${host}/events/api`;

  fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      name,
      event: { ...event, host, env, referer },
    }),
  });
};
