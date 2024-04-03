export enum TimeUnit {
  SECOND = "second",
  MINUTE = "minute",
  HOUR = "hour",
  DAY = "day",
  WEEK = "week",
  MONTH = "month",
  YEAR = "year",
}

export const timeUnitToSeconds = {
  [TimeUnit.SECOND]: 1,
  [TimeUnit.MINUTE]: 60,
  [TimeUnit.HOUR]: 60 * 60,
  [TimeUnit.DAY]: 60 * 60 * 24,
  [TimeUnit.WEEK]: 60 * 60 * 24 * 7,
  [TimeUnit.MONTH]: 60 * 60 * 24 * 30,
  [TimeUnit.YEAR]: 60 * 60 * 24 * 365,
};
