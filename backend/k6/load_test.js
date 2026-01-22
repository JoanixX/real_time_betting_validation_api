import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 20 },  // Fase de calentamiento (subimos a 20 usuarios)
    { duration: '1m', target: 50 },   // Fase de estrés (mantenemos 50 usuarios concurrentes)
    { duration: '20s', target: 0 },   // Fase de enfriamiento (bajamos a 0)
  ],
  thresholds: {
    // El 95% de las peticiones deben completarse en menos de 500ms
    // Si esto no se cumple, el test falla.
    http_req_duration: ['p(95)<500'], 
  },
};

export default function () {
  const res = http.get('http://localhost:8000/health_check');
  
  check(res, {
    'status fue 200': (r) => r.status == 200,
  });
  
  sleep(1);
}
