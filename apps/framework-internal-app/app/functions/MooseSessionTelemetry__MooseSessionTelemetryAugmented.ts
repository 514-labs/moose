// Import your Moose data models to use in the streaming function
import {
  MooseSessionTelemetry,
  MooseSessionTelemetryAugmented,
} from "../datamodels/mooseMetrics";
import { gatherIpData } from "../lib/ipAugmentation";

// The 'run' function transforms MooseSessionTelemetry data to MooseSessionTelemetryAugmented format.
// For more details on how Moose streaming functions work, see: https://docs.moosejs.com
export default async function run(
  source: MooseSessionTelemetry,
): Promise<MooseSessionTelemetryAugmented | null> {
  const telemetryEventAugmented: MooseSessionTelemetryAugmented = {
    ...source,
    ...(await gatherIpData(source.ip)),
  };

  // Unix Timestamp was sent through on the previous version of the CLI
  // but the JS date object takes milliseconds, so we need to multiply by 1000
  if (typeof source.timestamp === "number") {
    telemetryEventAugmented.timestamp = new Date(source.timestamp * 1000);
  }

  return telemetryEventAugmented;
}
