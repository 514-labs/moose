import http from "k6/http";
import { sleep, check } from "k6";

export const options = {
  discardResponseBodies: true,
  scenarios: {
    contacts: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: "60s", target: 10000 },
        { duration: "60s", target: 10000 },
      ],
      gracefulRampDown: "30s",
    },
  },
};
//
// export const options = {
//   vus: 1,
//   duration: "1s",
// };

// The function that defines VU logic.
//
// See https://grafana.com/docs/k6/latest/examples/get-started-with-k6/ to learn more
// about authoring k6 scripts.
//
export default function () {
  let res = http.get("http://[::1]:4001/dailyActiveUsers");
  check(res, { "status was 200": (r) => r.status == 200 });

  sleep(0.25);
}
