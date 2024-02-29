import { NextRequest } from "next/server";

// send an event to mixpanel
export async function POST(request: NextRequest) {
    const ip = request.ip ? request.ip : request.headers.get('X-Forwarded-For');
    
    const body = await request.json();

    const Mixpanel = require('mixpanel');
    const mixpanel = Mixpanel.init('be8ca317356e20c587297d52f93f3f9e');

    const event = {ip, ...body.event};

    mixpanel.track(body.name, event);

    return new Response("OK");
}