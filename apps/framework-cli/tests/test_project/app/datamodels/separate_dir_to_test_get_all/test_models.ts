type Key<T extends string | number> = T;

export interface Awesome {
  id: Key<number>;
  name: string;
  description: string;
}

export interface User {
  id: Key<number>;
  email: string;
  name?: string;
}