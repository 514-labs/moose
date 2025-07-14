import { Key } from "@514labs/moose-lib";

export interface PrimaryKeyOnlyModel {
  id: Key<string>; // Primary key
  name: string;
  timestamp: number;
  value: number;
}

export const PrimaryKeyOnlyTable = {
  name: "primary_key_only_table",
  // No orderByFields specified - should fall back to primary key 'id'
};
