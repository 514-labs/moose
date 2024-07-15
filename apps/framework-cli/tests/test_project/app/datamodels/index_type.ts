type Key<T extends string | number> = T;

export interface MyModel {
  name: Key<string>;
  custom_properties: {
    [key: string]: any;
  }
}
