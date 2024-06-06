type Key<T extends string | number> = T;

export interface Base {
  id: Key<string>;
}

interface User extends Base {
  name: string;
  email: string;
}
