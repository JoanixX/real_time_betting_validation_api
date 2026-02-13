'use client';

import { useEffect, useCallback, useState } from 'react';
import { getSocketClient, type ConnectionState } from '@/lib/socket';
import { useOddsStore } from '@/store/odds-store';
import { useBettingStore } from '@/store/betting-store';

interface UseSocketOptions {
  autoConnect?: boolean;
}

// hook central que conecta el WebSocket y routea eventos a los stores
export function useSocket(options: UseSocketOptions = {}) {
  const { autoConnect = true } = options;
  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected');

  const updateOdds = useOddsStore((s) => s.updateOdds);
  const resolveBet = useBettingStore((s) => s.resolveBet);

  const connect = useCallback(() => {
    getSocketClient().connect();
  }, []);

  const disconnect = useCallback(() => {
    getSocketClient().disconnect();
  }, []);

  useEffect(() => {
    const client = getSocketClient();

    // estado de conexión
    const unsubState = client.onStateChange(setConnectionState);

    // pipeline WS va a zustand y actualiza odds
    const unsubOdds = client.on('odds:updated', (payload) => {
      updateOdds(payload.match_id, payload.odds);
    });

    // pipeline WS va a zustand y actualiza apuesta confirmada
    const unsubValidated = client.on('bet:validated', (payload) => {
      resolveBet(payload.bet_id, 'Validated');
    });

    // pipeline WS va a zustand y actualiza apuesta rechazada
    const unsubRejected = client.on('bet:rejected', (payload) => {
      resolveBet(payload.bet_id, 'Rejected');
    });

    if (autoConnect) {
      client.connect();
    }

    return () => {
      unsubState();
      unsubOdds();
      unsubValidated();
      unsubRejected();
    };
  }, [autoConnect, updateOdds, resolveBet]);

  return { connectionState, isConnected: connectionState === 'connected', connect, disconnect };
}
