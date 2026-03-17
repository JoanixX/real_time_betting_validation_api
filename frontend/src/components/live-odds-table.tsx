'use client';

import {
  Table,
  TableBody,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { LiveOddsRow } from '@/components/live-odds-row';
import type { Match } from '@/types/domain';

interface LiveOddsTableProps {
  matches: Match[];
}

// La tabla wrapper solo re-renderiza cuando la lista de matches cambia
// las actualizaciones de odds individuales son manejadas por liveoddsrow via Zustand
export function LiveOddsTable({ matches }: LiveOddsTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-[200px]">Partido</TableHead>
          <TableHead className="w-[100px]">Estado</TableHead>
          <TableHead className="w-[100px] text-right">Cuota</TableHead>
          <TableHead className="w-[100px] text-right">Acción</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {matches.map((match) => (
          <LiveOddsRow
            key={match.id}
            matchId={match.id}
            homeTeam={match.home_team}
            awayTeam={match.away_team}
            status={match.status}
          />
        ))}
      </TableBody>
    </Table>
  );
}