import { Base } from "./extend.m.ts";

export interface User extends Base {
    name: string;
    email: string;
}