import { IngestPipeline, OlapTable } from "@514labs/moose-lib";
import { WatchEventWithRepo } from "./datamodels/WatchEvent";

export const watchEventPipeline = new OlapTable<WatchEventWithRepo>(
  "watch-event",
);
