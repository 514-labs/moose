type Key<T extends string | number> = T;

enum StringedEnum {
  A = "TEST",
}

interface MyModel {
  name: Key<string>;
  age: number;
  // abc: MyEnum;
  flag: boolean;
  "test-key": string;
  arr: string[];
  opt?: string;
  enum3: StringedEnum;
}
