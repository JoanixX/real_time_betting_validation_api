'use client';

import { create } from 'zustand';
import type { ActivityLogEntry, BetTicket, BetStatus } from '@/types/domain';
import { ACTIVITY_LOG_MAX_ENTRIES } from '@/lib/constants';

// apuesta pendiente, que va a esperar confirmación WS
interface PendingBet {
  bet_id: string;
  ticket: BetTicket;
  submittedAt: number;
}

interface BettingState {
  // Estado
  lastBet: BetTicket | null;
  activityLog: ActivityLogEntry[];
  isOnline: boolean;
  pendingBets: Map<string, PendingBet>;

  // Acciones
  addLogEntry: (entry: ActivityLogEntry) => void;
  setLastBet: (bet: BetTicket) => void;
  setOnline: (status: boolean) => void;
  clearLog: () => void;

  // Flujo de apuesta: REST envía, se vuelvepending, 
  // el WS confirma o rechaza y se envía el log
  addPendingBet: (betId: string, ticket: BetTicket) => void;
  resolveBet: (betId: string, status: BetStatus) => void;
}

export const useBettingStore = create<BettingState>((set) => ({
  lastBet: null,
  activityLog: [],
  isOnline: false,
  pendingBets: new Map(),

  addLogEntry: (entry) => set((state) => ({
    activityLog: [
      entry,
      ...state.activityLog,
    ].slice(0, ACTIVITY_LOG_MAX_ENTRIES),
  })),

  setLastBet: (bet) => set({ lastBet: bet }),

  setOnline: (status) => set({ isOnline: status }),

  clearLog: () => set({ activityLog: [] }),

  // se agrega apuesta a pending mientras hasta que se confirme el WS
  addPendingBet: (betId, ticket) =>
    set((state) => {
      const next = new Map(state.pendingBets);
      next.set(betId, { bet_id: betId, ticket, submittedAt: Date.now() });
      return { pendingBets: next };
    }),

  // WS confirma o rechaza y sacamos de pending y logueamos
  resolveBet: (betId, status) =>
    set((state) => {
      const next = new Map(state.pendingBets);
      const pending = next.get(betId);
      next.delete(betId);

      if (!pending) return { pendingBets: next };

      const latency = Date.now() - pending.submittedAt;
      return {
        pendingBets: next,
        lastBet: pending.ticket,
        activityLog: [
          {
            timestamp: new Date().toISOString(),
            amount: pending.ticket.amount,
            latency_ms: latency,
            status,
          },
          ...state.activityLog,
        ].slice(0, ACTIVITY_LOG_MAX_ENTRIES),
      };
    }),
}));
