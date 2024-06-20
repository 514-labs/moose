type Key<T extends string | number> = T;

enum NumEnum {
  A = 1,
}

enum StringedEnum {
  A = "TEST",
}

enum IterationEnum {
  A,
  B = 10,
  C,
}

export interface MyModel {
  name: Key<string>;
  age: number;
  // abc: MyEnum;
  flag: boolean;
  "test-key": string;
  arr: string[];
  opt?: string;
  enum3: StringedEnum;
  enum2: IterationEnum;
  enum1: NumEnum;
  logs: {
    info: string;
  }
}
