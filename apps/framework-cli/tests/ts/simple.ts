type Key<T extends string | number> = T;

enum MyEnum {
  A,
  B,
  C,
}

interface MyModel {
  name: Key<string>;
  age: number;
  abc: MyEnum;
  flag: boolean;
  "test-key": string;
  opt?: string;
}
