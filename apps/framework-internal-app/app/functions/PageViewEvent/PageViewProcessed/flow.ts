// Import your Moose data models to use in the streaming function
import { PageViewEvent, PageViewProcessed } from "../../../datamodels/models";
import { gatherIpData } from "../../../lib/ipAugmentation";

// The 'run' function transforms MooseActivityRaw data to MooseActivity format.
// For more details on how Moose flows work, see: https://docs.moosejs.com
export default async function run(
  source: PageViewEvent,
): Promise<PageViewProcessed | null> {
  const pageViewEvent: PageViewProcessed = {
    ...source,
    ...(await gatherIpData(source.ip)),
  };

  // Unix Timestamp was sent through on the previous version of the CLI
  // but the JS date object takes milliseconds, so we need to multiply by 1000
  if (typeof source.timestamp === "number") {
    pageViewEvent.timestamp = new Date(source.timestamp * 1000);
  }

  return pageViewEvent;
}
