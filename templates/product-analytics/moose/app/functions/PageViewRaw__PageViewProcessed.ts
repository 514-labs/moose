import { PageViewRaw, PageViewProcessed } from "../datamodels/models";
import { UAParser } from "ua-parser-js";

export default function run(event: PageViewRaw): PageViewProcessed {
  console.log(`Processing event: ${JSON.stringify(event)}`);
  // Process the user_agent to extract device, browser, and OS details
  const { browser, device, os } = UAParser(event.user_agent);

  // Extract the hostname from the URL
  const hostname = new URL(event.href).hostname;

  return {
    ...event,
    hostname: hostname,
    device_vendor: device.vendor || "",
    device_type: device.type || "desktop",
    device_model: device.model || "",
    browser_name: browser.name || "",
    browser_version: browser.version || "",
    os_name: os.name || "",
    os_version: os.version || "",
  };
}
