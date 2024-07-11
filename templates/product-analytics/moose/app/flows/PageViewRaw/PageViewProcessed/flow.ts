// Add your models & start the development server to import these types
import { PageViewProcessed, PageViewRaw } from "../../../datamodels/models";
// The 'run' function transforms PageViewRaw data to PageViewProcessed format.
// For more details on how Moose flows work, see: https://docs.moosejs.com

export default function run(event: PageViewRaw): PageViewProcessed | null {
  const ua = event.user_agent.toLowerCase();

  const deviceType = /mobile|tablet/.test(ua) ? "mobile" : "desktop";

  let deviceManufacturer = "Unknown";
  let deviceOperatingSystem = "Unknown";
  let deviceOSVersion = "Unknown";

  if (/android/.test(ua)) {
    deviceManufacturer = "Android";
    deviceOperatingSystem = "Android";
    const match = ua.match(/android\s([\d\.]+)/);
    deviceOSVersion = match ? match[1] : "Unknown";
  } else if (/iphone|ipad|ipod/.test(ua)) {
    deviceManufacturer = "Apple";
    deviceOperatingSystem = "iOS";
    const match = ua.match(/os\s([\d_]+)/);
    deviceOSVersion = match ? match[1].replace(/_/g, ".") : "Unknown";
  } else if (/windows/.test(ua)) {
    deviceManufacturer = "Microsoft";
    deviceOperatingSystem = "Windows";
    const match = ua.match(/windows\snt\s([\d\.]+)/);
    deviceOSVersion = match ? match[1] : "Unknown";
  } else if (/macintosh|mac os x/.test(ua)) {
    deviceManufacturer = "Apple";
    deviceOperatingSystem = "macOS";
    const match = ua.match(/mac\sos\sx\s([\d_]+)/);
    deviceOSVersion = match ? match[1].replace(/_/g, ".") : "Unknown";
  } else if (/linux|unix|bsd/.test(ua)) {
    deviceManufacturer = "Linux";
    deviceOperatingSystem = "Linux";
    const match = ua.match(/linux|unix|bsd\s([\d\.]+)/);
    deviceOSVersion = match ? match[1] : "Unknown";
  }

  let browserType = "Unknown";
  let browserVersion = "Unknown";
  if (/chrome|crios/.test(ua)) {
    browserType = "Chrome";
    const match = ua.match(/chrome\/([\d\.]+)/);
    browserVersion = match ? match[1] : "Unknown";
  } else if (/firefox/.test(ua)) {
    browserType = "Firefox";
    const match = ua.match(/firefox\/([\d\.]+)/);
    browserVersion = match ? match[1] : "Unknown";
  } else if (/opera/.test(ua)) {
    browserType = "Opera";
    const match = ua.match(/opera\/([\d\.]+)/);
    browserVersion = match ? match[1] : "Unknown";
  } else if (/safari/.test(ua)) {
    browserType = "Safari";
    const match = ua.match(/version\/([\d\.]+)/);
    browserVersion = match ? match[1] : "Unknown";
  } else if (/edge/.test(ua)) {
    browserType = "Edge";
    const match = ua.match(/edge\/([\d\.]+)/);
    browserVersion = match ? match[1] : "Unknown";
  } else if (/msie|trident/.test(ua)) {
    browserType = "IE";
    const match = ua.match(/msie ([\d\.]+)/);
    browserVersion = match ? match[1] : "Unknown";
  }

  const pattern =
    /www\.moosejs\.com|www\.fiveonefour\.com|www\.docs\.moosejs\.com/;

  const hostname = pattern.test(event.href)
    ? pattern.exec(event.href)?.[0]
    : "development";

  return {
    eventId: event.eventId,
    timestamp: event.timestamp,
    session_id: event.session_id,
    locale: event.locale,
    location: event.location,
    href: event.href,
    hostname: hostname || "development",
    pathname: event.pathname,
    referrer: event.referrer,
    device_manufacturer: deviceManufacturer,
    os: deviceOperatingSystem,
    os_version: deviceOSVersion,
    browser_type: browserType,
    browser_version: browserVersion,
  };
}
