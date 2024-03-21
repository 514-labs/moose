import http from "k6/http";
import { check, sleep } from "k6";

export default function () {
  const url = "http://[::1]:4000/ingest/UserActivity/0.0";
  const eventId = Math.floor(new Date().getTime());
  const userId = Math.floor(Math.random() * Number.MAX_SAFE_INTEGER).toString(
    36
  );

  const payload = JSON.stringify({
    eventId,
    userId,
    timestamp: new Date().toISOString(),
    activity: "loadtest",
  });
  const params = {
    headers: {
      "Content-Type": "application/json",
    },
  };

  let res = http.post(url, payload, params);
  check(res, { "status was 200": (r) => r.status == 200 });

  sleep(0.25);
}
