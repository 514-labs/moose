import { NextRequest } from "next/server";
import { sendServerEvent } from "event-capture/server-event";

// send an event to mixpanel
export async function POST(request: NextRequest) {
  console.log("framework-landing event route POST");
  const { name, ...payload } = await request.json();
  await sendServerEvent(name, payload);
  return new Response(null, { status: 204 });
}
