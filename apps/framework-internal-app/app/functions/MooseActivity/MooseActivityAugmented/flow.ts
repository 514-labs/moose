// Import your Moose data models to use in the streaming function
import {
  MooseActivity,
  MooseActivityAugmented,
} from "../../../datamodels/mooseMetrics";
import { gatherIpData } from "../../../lib/ipAugmentation";

// The 'run' function transforms MooseActivityRaw data to MooseActivity format.
// For more details on how Moose flows work, see: https://docs.moosejs.com
export default async function run(
  source: MooseActivity,
): Promise<MooseActivityAugmented | null> {
  const mooseActivity: MooseActivityAugmented = {
    ...source,
    ...(await gatherIpData(source.ip)),
  };

  // Unix Timestamp was sent through on the previous version of the CLI
  // but the JS date object takes milliseconds, so we need to multiply by 1000
  if (typeof source.timestamp === "number") {
    mooseActivity.timestamp = new Date(source.timestamp * 1000);
  }

  return mooseActivity;
}
