// Add your models & start the development server to import these types
import {
  MooseActivity,
  MooseActivityAugmented,
} from "../../../datamodels/models";
import process from "node:process";
import crypto from "node:crypto";

const IP_INFO_API_KEY = process.env["IP_INFO_API_KEY"];

if (!IP_INFO_API_KEY) {
  throw new Error("IP_INFO_API_KEY is required");
}

// The 'run' function transforms MooseActivityRaw data to MooseActivity format.
// For more details on how Moose flows work, see: https://docs.moosejs.com
export default async function run(
  source: MooseActivity,
): Promise<MooseActivityAugmented | null> {
  const mooseActivity: MooseActivityAugmented = {
    ...source,
  };

  // This is adding a comment

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

    mooseActivity.hashedIp = hash.digest("hex");
    mooseActivity.cityName = ipData?.city;
    mooseActivity.countryCode = ipData?.country;
    mooseActivity.companyName = ipData?.company.name;
    mooseActivity.companyType = ipData?.company.type;
    mooseActivity.companyDomain = ipData?.company.domain;
  }

  return mooseActivity;
}
