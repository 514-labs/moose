// Add your models & start the development server to import these types
import { PageViewEvent, PageViewProcessed } from "../../../datamodels/models";
import process from "node:process";
import crypto from "node:crypto";

const IP_INFO_API_KEY = process.env["IP_INFO_API_KEY"];

if (!IP_INFO_API_KEY) {
  throw new Error("IP_INFO_API_KEY is required");
}

// The 'run' function transforms MooseActivityRaw data to MooseActivity format.
// For more details on how Moose flows work, see: https://docs.moosejs.com
export default async function run(
  source: PageViewEvent,
): Promise<PageViewProcessed | null> {
  const pageViewEvent: PageViewProcessed = {
    ...source,
  };

  // Unix Timestamp was sent through on the previous version of the CLI
  // but the JS date object takes milliseconds, so we need to multiply by 1000
  if (typeof source.timestamp === "number") {
    pageViewEvent.timestamp = new Date(source.timestamp * 1000);
  }

  const ip = source.ip;

  if (ip) {
    const ipRes = await fetch(
      `https://ipinfo.io/${ip}/json?token=${IP_INFO_API_KEY}`,
    );
    const ipData = (await ipRes.json()) as
      | {
          city: string;
          country: string;
          company: {
            name: string;
            type: string;
            domain: string;
          };
        }
      | undefined;

    const hash = crypto.createHash("sha256");
    hash.update(ip);

    pageViewEvent.hashedIp = hash.digest("hex");
    pageViewEvent.cityName = ipData?.city;
    pageViewEvent.countryCode = ipData?.country;
    pageViewEvent.companyName = ipData?.company.name;
    pageViewEvent.companyType = ipData?.company.type;
    pageViewEvent.companyDomain = ipData?.company.domain;
  }

  return pageViewEvent;
}
