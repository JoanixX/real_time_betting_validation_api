'use client';

import { useQuery } from '@tanstack/react-query';
import { fetchBetHistory, fetchCurrentUser } from '@/lib/api';
import { QUERY_STALE_TIMES } from '@/components/providers';

// historial de apuestas (data fria, cacheada 5 min)
// se invalida automáticamente cuando usePlaceBet tiene éxito
export function useUserHistory(userId: string | null) {
  return useQuery({
    queryKey: ['bet-history', userId],
    queryFn: () => fetchBetHistory(userId!),
    enabled: !!userId,
    staleTime: QUERY_STALE_TIMES.BET_HISTORY,
  });
}

// perfil del usuario (data muy fria porque nos e edita casi nunca, staleTime infinito)
// solo se refresca al hacer login o editar perfil
export function useCurrentUser() {
  return useQuery({
    queryKey: ['current-user'],
    queryFn: fetchCurrentUser,
    staleTime: QUERY_STALE_TIMES.USER_PROFILE,
    retry: false,
  });
}