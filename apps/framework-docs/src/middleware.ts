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
    "/((?!api|_next/static|images|favicon.ico).*)",
  ],
};
export async function middleware(request: NextRequest) {
  const ip = request.ip ? request.ip : request.headers.get("X-Forwarded-For");
  const host = request.nextUrl.host;
  const referrer = headers().get("referer") ?? request.referrer;
  const pathname = request.nextUrl.pathname;

  const env = process.env.NODE_ENV;
  fetch(`${request.nextUrl.origin}/api/event`, {
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
  });

  return NextResponse.next();
}
