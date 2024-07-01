import { NextRequest } from "next/server";
import { sendServerEvent } from "@514labs/event-capture/server-event";

// send an event to mixpanel
export async function POST(request: NextRequest) {
  const { eventName, ...payload } = await request.json();
  try {
    await sendServerEvent(eventName, payload);
  } catch (error) {
    console.error(error);
  }
  return new Response("OK");
}
