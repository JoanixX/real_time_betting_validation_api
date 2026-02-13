'use client';

import { create } from 'zustand';

export interface BetSelection {
  matchId: string;
  homeTeam: string;
  awayTeam: string;
  odds: number; // odds al momento de seleccionar
  amount: number; // monto apostado
}

interface SelectionsState {
  selections: Map<string, BetSelection>;

  addSelection: (selection: BetSelection) => void;
  removeSelection: (matchId: string) => void;
  updateAmount: (matchId: string, amount: number) => void;

  // Actualizar odds (cuando cambia en real-time y el usuario aún no envió)
  syncOdds: (matchId: string, odds: number) => void;
  clearAll: () => void;
}

export const useSelectionsStore = create<SelectionsState>((set) => ({
  selections: new Map(),
  // funciones
  addSelection: (selection) =>
    set((state) => {
      const next = new Map(state.selections);
      next.set(selection.matchId, selection);
      return { selections: next };
    }),

  removeSelection: (matchId) =>
    set((state) => {
      const next = new Map(state.selections);
      next.delete(matchId);
      return { selections: next };
    }),

  updateAmount: (matchId, amount) =>
    set((state) => {
      const next = new Map(state.selections);
      const existing = next.get(matchId);
      if (existing) {
        next.set(matchId, { ...existing, amount });
      }
      return { selections: next };
    }),

  syncOdds: (matchId, odds) =>
    set((state) => {
      const next = new Map(state.selections);
      const existing = next.get(matchId);
      if (existing) {
        next.set(matchId, { ...existing, odds });
      }
      return { selections: next };
    }),

  clearAll: () => set({ selections: new Map() }),
}));