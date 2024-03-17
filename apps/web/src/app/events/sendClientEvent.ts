export const sendClientEvent = async (
  name: string,
  identifier: string,
  event: any
) => {
  await fetch("/events/api", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      name,
      event: { ...event, ip: identifier },
    }),
  });
};
