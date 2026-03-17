'use client';

import { useQuery } from '@tanstack/react-query';
import { fetchActiveMatches } from '@/lib/api';
import { useOddsStore } from '@/store/odds-store';
import { useEffect } from 'react';

// hook para el snapshot inicial de partidos, este llama a la API
//  y cuando tiene data llena el odds-store
export function useActiveMatches() {
  const query = useQuery({
    queryKey: ['matches', 'active'],
    queryFn: fetchActiveMatches,
    // el staletime se puede ajustar
    staleTime: 10_000, 
  });

  const updateOddsBatch = useOddsStore((s) => s.updateOddsBatch);

  // llenamos el store del zustand cuando llega info nueva
  useEffect(() => {
    if (query.data) {
      const updates = query.data.map((m) => ({
        match_id: m.id,
        odds: m.odds,
      }));
      updateOddsBatch(updates);
    }
  }, [query.data, updateOddsBatch]);

  return query;
}