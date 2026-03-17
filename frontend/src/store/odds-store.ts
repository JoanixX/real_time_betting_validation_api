'use client';

import { create } from 'zustand';

// mapa de odds
interface OddsEntry {
  odds: number;
  lastUpdated: number; // es un Date.now() timestamp
}

interface OddsState {
  // Estado
  // usamos un record plano para O(1) en los lookups y comparaciones por referencia
  odds: Record<string, OddsEntry>;

  // Acciones
  updateOdds: (matchId: string, odds: number) => void;
  updateOddsBatch: (updates: Array<{ match_id: string; odds: number }>) => void;
  removeMatch: (matchId: string) => void;
  clear: () => void;
}

export const useOddsStore = create<OddsState>((set) => ({
  odds: {},

  updateOdds: (matchId, odds) =>
    set((state) => ({
      odds: {
        ...state.odds,
        [matchId]: { odds, lastUpdated: Date.now() },
      },
    })),

  updateOddsBatch: (updates) =>
    set((state) => {
      const now = Date.now();
      const newOdds = { ...state.odds };
      for (const u of updates) {
        newOdds[u.match_id] = { odds: u.odds, lastUpdated: now };
      }
      return { odds: newOdds };
    }),

  removeMatch: (matchId) =>
    set((state) => {
      const newOdds = { ...state.odds };
      delete newOdds[matchId];
      return { odds: newOdds };
    }),

  clear: () => set({ odds: {} }),
}));

// se usa el selector helper para leer odds de un match específico
// la re-renderizacion granular es O(1) con zustand
export function useMatchOdds(matchId: string): number | undefined {
  return useOddsStore((s) => s.odds[matchId]?.odds);
}

// selector para saber la fecha de ultima actualizacion
export function useMatchOddsTimestamp(matchId: string): number | undefined {
  return useOddsStore((s) => s.odds[matchId]?.lastUpdated);
}