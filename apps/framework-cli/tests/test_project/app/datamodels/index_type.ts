type Key<T extends string | number> = T;

export interface MyModel {
  name: Key<string>;
  [key: symbol]: any;
}
