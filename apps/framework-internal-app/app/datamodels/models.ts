import { Key, IngestionFormat } from "@514labs/moose-lib";
import { IpAugmentation } from "../lib/ipAugmentation";

export const PageViewEventConfig = {
  storage: {
    enabled: false,
  },
};

export interface PageViewEvent {
  eventId: Key<string>;
  timestamp: Date;
  session_id: string;
  user_agent: string;
  locale: string;
  location: string;
  href: string;
  pathname: string;
  referrer: string;
  ip?: string;
}

export interface PageViewProcessed
  extends IpAugmentation,
    Omit<PageViewEvent, "ip"> {}

export interface TrackEvent extends PageViewEvent {
  action: string;
  name: string;
  subject: string;
  targetUrl?: string;
}

export interface ParsedActivity {
  eventId: Key<string>;
  timestamp: Date;
  userId: string;
  activity: string;
}

export interface UserActivity {
  eventId: Key<string>;
  timestamp: Date;
  userId: string;
  activity: string;
  description: string;
}

export const UserActivityBatchConfig = {
  ingestion: {
    format: IngestionFormat.JSON_ARRAY,
  },
};

export interface UserActivityBatch {
  id: Key<string>;
  timestamp: Date;
  userId: string;
  activity: string;
  description: string;
}

export const BigBatchConfig = {
  ingestion: {
    format: IngestionFormat.JSON_ARRAY,
  },
};

export interface BigBatch {
  id: Key<string>;
  test_column_1: string;
  test_column_2: string;
  test_column_3: string;
  test_column_4: string;
  test_column_5: string;
  test_column_6: string;
  test_column_7: string;
  test_column_8: string;
  test_column_9: string;
  test_column_10: string;
  test_column_11: number;
  test_column_12: number;
  test_column_13: number;
  test_column_14: number;
  test_column_15: number;
  test_column_16: number;
  test_column_17: number;
  test_column_18: number;
  test_column_19: number;
  test_column_20: number;
  test_column_21: number;
  test_column_22: number;
  test_column_23: number;
  test_column_24: number;
  test_column_25: number;
  test_column_26: number;
  test_column_27: number;
  test_column_28: number;
  test_column_29: number;
  test_column_30: number;
  test_column_31: number;
  test_column_32: number;
  test_column_33: number;
  test_column_34: number;
  test_column_35: number;
  test_column_36: number;
  test_column_37: number;
  test_column_38: number;
  test_column_39: number;
  test_column_40: number;
  test_column_41: number;
  test_column_42: number;
  test_column_43: number;
  test_column_44: number;
  test_column_45: number;
  test_column_46: number;
  test_column_47: number;
  test_column_48: number;
  test_column_49: number;
  test_column_50: number;
  test_column_51: number;
  test_column_52: number;
  test_column_53: number;
  test_column_54: number;
  test_column_55: number;
  test_column_56: number;
  test_column_57: number;
  test_column_58: number;
  test_column_59: number;
  test_column_60: number;
  test_column_61: number;
  test_column_62: number;
}
