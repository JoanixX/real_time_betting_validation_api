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
  selection: BetSelection | null;
  isSubmitting: boolean;

  setSelection: (selection: BetSelection) => void;
  clearSelection: () => void;
  updateAmount: (amount: number) => void;
  setSubmitting: (isSubmitting: boolean) => void;

  // Actualizar odds (cuando cambia en real-time y el usuario aun no envio)
  syncOdds: (matchId: string, odds: number) => void;
}

export const useSelectionsStore = create<SelectionsState>((set) => ({
  selection: null,
  isSubmitting: false,

  setSelection: (selection) => set({ selection }),

  clearSelection: () => set({ selection: null }),

  updateAmount: (amount) =>
    set((state) => ({
      selection: state.selection ? { ...state.selection, amount } : null,
    })),

  setSubmitting: (isSubmitting) => set({ isSubmitting }),

  syncOdds: (matchId, odds) =>
    set((state) => {
      if (state.selection && state.selection.matchId === matchId) {
        return { selection: { ...state.selection, odds } };
      }
      return state;
    }),
}));