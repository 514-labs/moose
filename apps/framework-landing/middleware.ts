import { NextRequest, NextResponse, userAgent } from "next/server";
import { cookies, headers } from "next/headers";
import { setPageViewEventProperties } from "@514labs/event-capture/server-event";

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    {
      source:
        "/((?!monitoring-tunnel|api|_next/static|images|favicon.ico|opengraph-image.*|wp-*|apple-*|cgi-*|robots.txt).*)",
      missing: [
        { type: "header", key: "next-router-prefetch" },
        { type: "header", key: "purpose", value: "prefetch" },
      ],
    },
  ],
};
export async function middleware(request: NextRequest) {
  const ip = request.ip ? request.ip : request.headers.get("X-Forwarded-For");
  const href = request.nextUrl.href;
  const host = request.nextUrl.host;
  const referrer = headers().get("referer") ?? request.referrer;
  const pathname = request.nextUrl.pathname;
  const { isBot, device, os, browser } = userAgent(request);
  const env = process.env.NODE_ENV;

  const session_id = cookies().get("session-id")?.value || "unknown";

  // const funcPayload = commonProperties(request);

  // const payload = {
  //   eventId: crypto.randomUUID(),
  //   timestamp: new Date(),
  //   eventName: 'page_view',
  //   session_id,
  //   href,
  //   host,
  //   referrer,
  //   ip,
  //   pathname,
  //   isBot,
  //   deviceType: device.type,
  //   deviceVendor: device.vendor,
  //   deviceModel: device.model,
  //   osName: os.name,
  //   osVersion: os.version,
  //   browserName: browser.name,
  //   browserVersion: browser.version,
  // };

  const payload = {
    ...setPageViewEventProperties(request),
  };

  console.log("Page View: \n", payload);

  const mooseHost =
    env === "development"
      ? "http://localhost:4000"
      : "https://moosefood.514.dev";

  const oldUrl = `${request.nextUrl.protocol}//${request.nextUrl.host}/api/event`;
  const mooseUrl = `${mooseHost}/ingest/PageViewEvent`;
  // try {
  //   const response = await fetch(mooseUrl, {
  //     method: "POST",
  //     headers: {
  //       "Content-Type": "application/json",
  //     },
  //     body: JSON.stringify(payload),
  //   });

  //   if (!response.ok) {
  //     throw new Error(
  //       `Failed to send event: ${response.status}, ${response.url}`,
  //     );
  //   }
  // } catch (error) {
  //   console.error(error);
  // }
  // return NextResponse.next();
}
