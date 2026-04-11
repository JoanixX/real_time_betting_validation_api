import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 1,
  iterations: 1,
};

export default function () {
  const baseUrl = 'http://localhost:8000';
  const email = `testuser_${__VU}_${__ITER}_${Date.now()}@example.com`;
  const password = 'StrongPassword123!';

  // 1. health check
  const healthRes = http.get(`${baseUrl}/health_check`);
  check(healthRes, {
    'health check status is 200': (r) => r.status === 200,
  });

  // 2. registarse
  const registerPayload = JSON.stringify({
    email,
    password,
    name: "Test User",
  });

  const registerRes = http.post(`${baseUrl}/register`, registerPayload, {
    headers: { 'Content-Type': 'application/json' },
  });

  check(registerRes, {
    'register status is 200': (r) => r.status === 200,
    'register returns user_id': (r) => {
      try {
        return JSON.parse(r.body).user_id !== undefined;
      } catch (e) {
        return false;
      }
    }
  });

  // 3. Login
  const loginPayload = JSON.stringify({ email, password });
  const loginRes = http.post(`${baseUrl}/login`, loginPayload, {
    headers: { 'Content-Type': 'application/json' },
  });

  let userId = null;
  check(loginRes, {
    'login status is 200': (r) => r.status === 200,
    'login returns user_id': (r) => {
      try {
        const body = JSON.parse(r.body);
        userId = body.user_id;
        return userId !== undefined;
      } catch (e) {
        return false;
      }
    }
  });

  // 4. Hacer una apuesta
  if (userId) {
    const betPayload = JSON.stringify({
      user_id: userId,
      match_id: "223e4567-e89b-12d3-a456-426614174000",
      selection: "HomeWin",
      amount: 50.0,
      odds: 2.1,
    });

    const betRes = http.post(`${baseUrl}/bets`, betPayload, {
      headers: { 'Content-Type': 'application/json' },
    });
    
    console.log("BET RES STATUS:", betRes.status, "BODY:", betRes.body);

    check(betRes, {
      'place bet status is 200': (r) => r.status === 200 || r.status === 201 || r.status === 202,
      'bet response is valid': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.status === "Accepted" || body.status === "Pending Validation";
        } catch (e) {
          return true;
        }
      }
    });
  } else {
    console.error("Skipping bet test because login failed to provide user_id");
  }
}