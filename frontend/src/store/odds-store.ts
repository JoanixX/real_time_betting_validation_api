'use client';

import { create } from 'zustand';

// mapa de odds
interface OddsEntry {
  odds: number;
  lastUpdated: number; // es un Date.now() timestamp
}

interface OddsState {
  // Estado
  // se hace un mapa para reducir la complejidad, con un objeto plano con muchas keys
  oddsMap: Map<string, OddsEntry>;

  // Acciones
  updateOdds: (matchId: string, odds: number) => void;
  updateOddsBatch: (updates: Array<{ match_id: string; odds: number }>) => void;
  removeMatch: (matchId: string) => void;
  clear: () => void;
}

export const useOddsStore = create<OddsState>((set) => ({
  oddsMap: new Map(),

  // hacemos que el zustand solo re-renderiza si la referencia del Map cambia
  updateOdds: (matchId, odds) =>
    set((state) => {
      const next = new Map(state.oddsMap);
      next.set(matchId, { odds, lastUpdated: Date.now() });
      return { oddsMap: next };
    }),

  // Batch update para cuando llegan múltiples odds juntos
  // un solo re-render por cada cambio del mapa
  updateOddsBatch: (updates) =>
    set((state) => {
      const next = new Map(state.oddsMap);
      const now = Date.now();
      for (const u of updates) {
        next.set(u.match_id, { odds: u.odds, lastUpdated: now });
      }
      return { oddsMap: next };
    }),

  removeMatch: (matchId) =>
    set((state) => {
      const next = new Map(state.oddsMap);
      next.delete(matchId);
      return { oddsMap: next };
    }),

  clear: () => set({ oddsMap: new Map() }),
}));

// se usa el selector helper para leer odds de un match específico
// sin re-renders de otros matches
// por ejemplo con el const odds = useMatchOdds('match-123')
export function useMatchOdds(matchId: string): number | undefined {
  return useOddsStore((s) => s.oddsMap.get(matchId)?.odds);
}

// otro selector para saber la fecha de ultima actualización
export function useMatchOddsTimestamp(matchId: string): number | undefined {
  return useOddsStore((s) => s.oddsMap.get(matchId)?.lastUpdated);
}