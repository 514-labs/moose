type Key<T extends string | number> = T;

enum SimpleEnum {
  A,
  B,
}

enum Mapped {
  A = 1,
  B,
}

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
  enum1: SimpleEnum;
  enum2: Mapped;
  enum3: StringedEnum;
}
