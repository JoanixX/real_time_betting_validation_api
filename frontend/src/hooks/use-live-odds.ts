'use client';

import { useOddsStore, useMatchOdds, useMatchOddsTimestamp } from '@/store/odds-store';

// hook para leer odds de un match especifico
// usa selector granular: solo re-renderiza cuando cambian los odds de ESTE match
export function useLiveOdds(matchId: string) {
  const odds = useMatchOdds(matchId);
  const lastUpdated = useMatchOddsTimestamp(matchId);

  return {
    odds,
    lastUpdated,
    isStale: lastUpdated ? Date.now() - lastUpdated > 10_000 : true,
  };
}

// hook para leer todos los odds activos, por ejemplo para el overview
// re-renderiza con cualquier cambio de odds
export function useAllLiveOdds() {
  return useOddsStore((s) => s.oddsMap);
}
