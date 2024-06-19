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
  example: Key<number>
  name: {
    test:string
  }
}
