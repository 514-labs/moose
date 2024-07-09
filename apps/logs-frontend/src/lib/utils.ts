import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export enum SeverityLevel {
  INFO = "INFO",
  WARN = "WARN",
  ERROR = "ERROR",
  DEBUG = "DEBUG",
}

export const severityLevels = [
  SeverityLevel.ERROR,
  SeverityLevel.WARN,
  SeverityLevel.INFO,
  SeverityLevel.DEBUG,
];

export const severityLevelColors = {
  [SeverityLevel.INFO]: "bg-blue-200",
  [SeverityLevel.WARN]: "bg-yellow-200",
  [SeverityLevel.ERROR]: "bg-red-200",
  [SeverityLevel.DEBUG]: "bg-green-200",
};

export function getApiRoute() {
  if (process.env.NODE_ENV === "development") {
    return "http://localhost:4000";
  } else {
    return process.env.API_ROUTE;
  }
}
