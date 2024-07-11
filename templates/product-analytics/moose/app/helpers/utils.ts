export function getDefaultFrom() {
  const oneWeekAgo = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000);
  return Math.floor(oneWeekAgo.getTime() / 1000).toString();
}

export function getDefaultTo() {
  const now = new Date();
  return Math.floor(now.getTime() / 1000).toString();
}
