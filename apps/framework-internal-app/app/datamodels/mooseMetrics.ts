import { Key } from "@514labs/moose-lib";
import { IpAugmentation } from "../lib/ipAugmentation";

export const MooseActivityConfig = {
  storage: {
    enabled: false,
  },
};

export interface MooseActivity {
  id: Key<string>;
  project: string;
  activityType: string;
  sequenceId: string;
  timestamp: Date;
  cliVersion: string;
  isMooseDeveloper?: boolean;
  machineId: string;
  ip?: string;
}

export interface MooseActivityAugmented
  extends Omit<MooseActivity, "ip">,
    IpAugmentation {}

export const MooseSessionTelemetryConfig = {
  storage: {
    enabled: false,
  },
};

export interface MooseSessionTelemetry {
  timestamp: Key<Date>;
  machineId: string;
  sequenceId: string;
  project: string;
  cliVersion: string;
  sessionDurationInSec: number;
  isProd: boolean;
  isMooseDeveloper: boolean;
  ingestedEventsCount: number;
  ingestedEventsTotalBytes: number;
  ingestAvgLatencyInMs: number;
  consumedRequestCount: number;
  consumedAvgLatencyInMs: number;
  //   blocksCount: number;
  streamingToOLAPEventSyncedCount: number;
  streamingToOLAPEventSyncedBytesCount: number;
  streamingFunctionsInputEventsProcessedCount: number;
  streamingFunctionsOutputEventsProcessedCount: number;
  streamingFunctionsEventsProcessedTotalBytes: number;
  ip?: string;
}

export interface MooseSessionTelemetryAugmented
  extends Omit<MooseSessionTelemetry, "ip">,
    IpAugmentation {}
