import { NextRequest } from "next/server";
import { sendServerEvent } from "event-capture/server-event";

// send an event to mixpanel
export async function POST(request: NextRequest) {
  const { name, ...payload } = await request.json();
  sendServerEvent(name, payload);
  return new Response("OK");
}
