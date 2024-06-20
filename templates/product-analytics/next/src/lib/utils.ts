import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function createMultiSelectOptions(options: string[]) {
  return options.map((option) => ({
    label: option,
    val: option,
  }));
}
