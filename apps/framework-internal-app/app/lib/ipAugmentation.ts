import * as process from "process";
import * as crypto from "crypto";

const IP_INFO_API_KEY = process.env["IP_INFO_API_KEY"];

if (!IP_INFO_API_KEY) {
  throw new Error("IP_INFO_API_KEY is required");
}

export type IpAugmentation = Partial<{
  hashedIp: string;
  cityName: string;
  countryCode: string;
  companyName: string;
  companyType: string;
  companyDomain: string;
}>;

export async function gatherIpData(ip?: string): Promise<IpAugmentation> {
  const toReturn: IpAugmentation = {};

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

    toReturn.hashedIp = hash.digest("hex");
    toReturn.cityName = ipData?.city;
    toReturn.countryCode = ipData?.country;
    toReturn.companyName = ipData?.company.name;
    toReturn.companyType = ipData?.company.type;
    toReturn.companyDomain = ipData?.company.domain;
  }

  return toReturn;
}
