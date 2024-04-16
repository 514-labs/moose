import { NextRequest, NextResponse } from "next/server";
import { headers } from "next/headers";

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    "/((?!monitoring-tunnel|api|_next/static|images|favicon.ico).*)",
  ],
};
export async function middleware(request: NextRequest) {
  const ip = request.ip ? request.ip : request.headers.get("X-Forwarded-For");
  const host = request.nextUrl.host;
  const referrer = headers().get("referer") ?? request.referrer;
  const pathname = request.nextUrl.pathname;

  const env = process.env.NODE_ENV;
  try {
    const response = await fetch(
      `${request.nextUrl.protocol}//${request.nextUrl.host}/api/event`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          name: "page_view",
          host,
          referrer,
          env,
          ip,
          pathname,
        }),
      },
    );
    console.log("URL", response.url);

    if (!response.ok) {
      throw new Error(
        `Failed to send event: ${response.statusText}, ${response.url}`,
      );
    }
  } catch (error) {
    console.error(error);
  }
  return NextResponse.next();
}
