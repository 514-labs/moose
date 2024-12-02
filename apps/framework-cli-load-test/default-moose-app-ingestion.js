import http from "k6/http";
import { sleep, check } from "k6";

const vues = 2000;
const duration = "30s";

export const options = {
  discardResponseBodies: true,
  cloud: {
    distribution: {
      distributionLabel1: { loadZone: "amazon:us:portland", percent: 100 },
    },
  },
  scenarios: {
    contacts: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: duration, target: vues },
        { duration: duration, target: vues },
      ],
      gracefulRampDown: "30s",
    },
  },
};

const data = {
  activity: "Login",
  timestamp: "2024-06-28T23:17:01+0000",
  eventId: "12344",
  userId: "test-value2",
};

export default function () {
  let res = http.post(
    "http://34.168.230.91:4000/ingest/UserActivity",
    JSON.stringify(data),
    {
      headers: { "Content-Type": "application/json" },
    },
  );
  check(res, { "status was 200": (r) => r.status == 200 });
  sleep(0.1);
}
