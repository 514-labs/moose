import { Key } from "@514labs/moose-lib";

export interface ExplicitOrderByModel {
  id: Key<string>; // Primary key
  name: string;
  timestamp: number;
  value: number;
}

export const ExplicitOrderByTable = {
  name: "explicit_order_by_table",
  orderByFields: ["timestamp", "name"], // Should use these, not 'id'
};
