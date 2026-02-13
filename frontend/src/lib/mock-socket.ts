import { useOddsStore } from '@/store/odds-store';
import type { MatchStatus } from '@/types/domain';

// Datos mock de partidos sin backend, solo simulacion
export interface MockMatch {
  id: string;
  home_team: string;
  away_team: string;
  status: MatchStatus;
  start_time: string;
  base_odds: number;
}

const MOCK_MATCHES: MockMatch[] = [
  { id: 'match-001', home_team: 'Real Madrid', away_team: 'FC Barcelona', status: 'live', start_time: new Date().toISOString(), base_odds: 1.85 },
  { id: 'match-002', home_team: 'Manchester City', away_team: 'Liverpool', status: 'live', start_time: new Date().toISOString(), base_odds: 2.10 },
  { id: 'match-003', home_team: 'Bayern Munich', away_team: 'Borussia Dortmund', status: 'live', start_time: new Date().toISOString(), base_odds: 1.65 },
  { id: 'match-004', home_team: 'PSG', away_team: 'Olympique Marseille', status: 'live', start_time: new Date().toISOString(), base_odds: 1.45 },
  { id: 'match-005', home_team: 'Juventus', away_team: 'AC Milan', status: 'live', start_time: new Date().toISOString(), base_odds: 2.35 },
  { id: 'match-006', home_team: 'Ajax', away_team: 'Feyenoord', status: 'upcoming', start_time: new Date().toISOString(), base_odds: 1.95 },
  { id: 'match-007', home_team: 'Boca Juniors', away_team: 'River Plate', status: 'live', start_time: new Date().toISOString(), base_odds: 2.50 },
  { id: 'match-008', home_team: 'Flamengo', away_team: 'Palmeiras', status: 'live', start_time: new Date().toISOString(), base_odds: 1.75 },
];

// Genera una variacion pseudoaleatoria de odds
// simula market microstructure: cambios pequeños y frecuentes
function jitterOdds(base: number): number {
  const change = (Math.random() - 0.5) * 0.15; // ±0.075
  const newOdds = base + change;
  return Math.max(1.01, parseFloat(newOdds.toFixed(2)));
}

let intervalId: ReturnType<typeof setInterval> | null = null;
const currentOdds: Map<string, number> = new Map();

// inicia la simulacion de odds — emite updates cada 100-300ms
export function startMockOddsStream(): () => void {
  const updateOdds = useOddsStore.getState().updateOdds;

  // inicializar odds base
  for (const match of MOCK_MATCHES) {
    currentOdds.set(match.id, match.base_odds);
    updateOdds(match.id, match.base_odds);
  }

  // emitir a intervalos aleatorios
  function scheduleNext() {
    const delay = 100 + Math.random() * 200; // 100-300ms
    intervalId = setTimeout(() => {
      // Elegir 1-3 matches aleatorios para actualizar
      const count = 1 + Math.floor(Math.random() * 3);
      for (let i = 0; i < count; i++) {
        const match = MOCK_MATCHES[Math.floor(Math.random() * MOCK_MATCHES.length)];
        if (match.status !== 'live') continue;

        const current = currentOdds.get(match.id) ?? match.base_odds;
        const newOdds = jitterOdds(current);
        currentOdds.set(match.id, newOdds);
        updateOdds(match.id, newOdds);
      }
      scheduleNext();
    }, delay);
  }
  scheduleNext();

  // retorna cleanup function
  return () => {
    if (intervalId) {
      clearTimeout(intervalId);
      intervalId = null;
    }
    currentOdds.clear();
  };
}

// getter de matches para la tabla
export function getMockMatches(): MockMatch[] {
  return MOCK_MATCHES;
}