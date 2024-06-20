export enum DateRange {
  "1H" = "1H",
  "Today" = "1D",
  "3D" = "3D",
  "7D" = "7D",
  "30D" = "30D",
}

export const rangeToNum = {
  [DateRange["1H"]]: 1,
  [DateRange.Today]: 1 * 24,
  [DateRange["3D"]]: 1 * 24 * 3,
  [DateRange["7D"]]: 1 * 24 * 7,
  [DateRange["30D"]]: 1 * 24 * 30,
};

// returns from and to date in UTC SECONDS
export function getRangeDate(range: DateRange) {
  const to = Math.floor(Date.now() / 1000);
  const from = to - rangeToNum[range] * 60 * 60;
  return { from, to };
}
