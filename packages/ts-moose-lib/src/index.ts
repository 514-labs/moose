export interface Aggregation {
  select: string;
  orderBy: string;
}

export type Key<T extends string | number | Date> = T;
