import { PageViewRaw, PageViewProcessed } from "../datamodels/models";
import { UAParser } from "ua-parser-js";

export function run(event: PageViewRaw): PageViewProcessed {
  // Process the user_agent to extract device, browser, and OS details
  const { browser, device, os } = UAParser(event.user_agent).getResult();

  // Extract the hostname from the URL
  const url = new URL(event.href);
  const hostname = url.hostname;

  return {
    eventId: event.eventId,
    timestamp: event.timestamp,
    session_id: event.session_id,
    locale: event.locale,
    location: event.location,
    href: event.href,
    hostname: hostname,
    pathname: event.pathname,
    referrer: event.referrer,
    device_vendor: device.vendor,
    device_type: device.type,
    device_model: device.model,
    browser_name: browser.name,
    browser_version: browser.version,
    browser_major: browser.major,
    os_name: os.name,
    os_version: os.version,
  };
}
