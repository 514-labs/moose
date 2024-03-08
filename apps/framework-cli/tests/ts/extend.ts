export interface Base {
    id: string;
}

type Key = string;

interface User extends Base {
    name: string; 
    email: string;
    id: Key;
}

type UserKey = User['id' & 'name'];