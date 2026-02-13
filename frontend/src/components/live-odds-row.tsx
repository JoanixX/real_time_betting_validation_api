'use client';

import React from 'react';
import { TableCell, TableRow } from '@/components/ui/table';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useLiveOdds } from '@/hooks/use-live-odds';
import { useOddsChange } from '@/hooks/use-odds-change';
import { useSelectionsStore } from '@/store/selections-store';
import { cn } from '@/lib/utils';
import { Plus, Check } from 'lucide-react';
import type { MatchStatus } from '@/types/domain';

interface LiveOddsRowProps {
  matchId: string;
  homeTeam: string;
  awayTeam: string;
  status: MatchStatus;
}

// fila memoizada, este solo re-renderiza cuando cambian los odds de ESTE match
// o cuando cambia su selección en el betting slip
const LiveOddsRow = React.memo(function LiveOddsRow({
  matchId,
  homeTeam,
  awayTeam,
  status,
}: LiveOddsRowProps) {
  const { odds } = useLiveOdds(matchId);
  const { direction, flashKey } = useOddsChange(odds);

  const isSelected = useSelectionsStore((s) => s.selections.has(matchId));
  const addSelection = useSelectionsStore((s) => s.addSelection);
  const removeSelection = useSelectionsStore((s) => s.removeSelection);

  const handleToggleSelection = () => {
    if (isSelected) {
      removeSelection(matchId);
    } else if (odds !== undefined) {
      addSelection({
        matchId,
        homeTeam,
        awayTeam,
        odds,
        amount: 10, // Monto default
      });
    }
  };

  const statusBadgeVariant = status === 'live'
    ? 'default'
    : status === 'suspended'
      ? 'destructive'
      : 'secondary';

  return (
    <TableRow
      key={flashKey}
      className={cn(
        'transition-colors',
        direction === 'up' && 'animate-flash-green',
        direction === 'down' && 'animate-flash-red',
      )}
    >
      <TableCell className="font-medium">
        <div className="flex flex-col">
          <span>{homeTeam}</span>
          <span className="text-xs text-muted-foreground">vs</span>
          <span>{awayTeam}</span>
        </div>
      </TableCell>

      <TableCell>
        <Badge variant={statusBadgeVariant} className="capitalize">
          {status === 'live' && (
            <span className="mr-1 inline-block h-2 w-2 animate-pulse rounded-full bg-green-400" />
          )}
          {status}
        </Badge>
      </TableCell>

      <TableCell className="text-right tabular-nums text-lg font-bold">
        <span
          className={cn(
            'transition-colors duration-200',
            direction === 'up' && 'text-green-500',
            direction === 'down' && 'text-red-500',
          )}
        >
          {odds?.toFixed(2) ?? '—'}
        </span>
      </TableCell>

      <TableCell className="text-right">
        <Button
          variant={isSelected ? 'default' : 'outline'}
          size="sm"
          onClick={handleToggleSelection}
          disabled={status !== 'live' || odds === undefined}
        >
          {isSelected ? (
            <>
              <Check className="mr-1 h-4 w-4" />
              Agregado
            </>
          ) : (
            <>
              <Plus className="mr-1 h-4 w-4" />
              Apostar
            </>
          )}
        </Button>
      </TableCell>
    </TableRow>
  );
});

export { LiveOddsRow };
export type { LiveOddsRowProps };