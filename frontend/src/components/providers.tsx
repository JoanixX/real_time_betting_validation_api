'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useState, type ReactNode } from 'react';

// staletime por tipo de data
export const QUERY_STALE_TIMES = {
  // refrescamos seguido
  HEALTH_CHECK: 30_000,
  //5 minutos, se invalida al colocar apuesta
  BET_HISTORY: 5 * 60 * 1_000,
  // perfil de usuario, que casi nunca cambia
  USER_PROFILE: Infinity,
} as const;

// provider de tanstack query, debe ser client component
export function QueryProvider({ children }: { children: ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // conservador
            refetchOnWindowFocus: false,
            retry: 1,
            staleTime: QUERY_STALE_TIMES.HEALTH_CHECK,
            // cache muerto se borra después de 10 min
            gcTime: 10 * 60 * 1_000,
          },
          mutations: {
            retry: 0,
          },
        },
      })
  );

  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}
