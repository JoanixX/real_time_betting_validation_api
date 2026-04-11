import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 50 },
    { duration: '1m', target: 200 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<50'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const url = 'http://localhost:8000/bets';

  const payload = JSON.stringify({
    user_id: "550e8400-e29b-41d4-a716-446655440000",
    match_id: "123e4567-e89b-12d3-a456-426614174000",
    selection: "HomeWin",
    amount: 10.50,
    odds: 1.85,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, payload, params);

  check(res, {
    'status is 200 or 201': (r) => r.status === 200 || r.status === 201,
    'latency is low': (r) => r.timings.duration < 100,
  });

  sleep(0.1);
}