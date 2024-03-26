export function humanReadableDate(publishedAt: string | Date) {
  return new Date(publishedAt).toLocaleDateString(undefined, {
    weekday: "short",
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}
