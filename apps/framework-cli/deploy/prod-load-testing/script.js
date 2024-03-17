import http from 'k6/http';
import { check, sleep } from 'k6';

// The function that defines VU logic.
//
// See https://grafana.com/docs/k6/latest/examples/get-started-with-k6/ to learn more
// about authoring k6 scripts.
//
export default function() {
  const url = 'http://34.82.14.129:4000/ingest/UserActivity';
  const eventId = Math.floor(Math.random() * 1e10);
  const userId = Math.floor(Math.random() * 1e6);
  
  const payload = JSON.stringify({
    eventId,
    userId,
    timestamp: new Date().toISOString(),
    activityType: "loadtest"
  });
  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  let res = http.post(url, payload, params);  
  check(res, { 'status was 200': (r) => r.status == 200 });

  sleep(0.25);
}
