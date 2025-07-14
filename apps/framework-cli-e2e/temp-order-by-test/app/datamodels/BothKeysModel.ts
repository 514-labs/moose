import { Key } from "@514labs/moose-lib";

export interface BothKeysModel {
  id: Key<string>; // Primary key
  user_id: Key<string>; // Another primary key
  name: string;
  timestamp: number;
  value: number;
}

export const BothKeysTable = {
  name: "both_keys_table",
  orderByFields: ["value", "timestamp"], // Should use these, not primary keys
};
