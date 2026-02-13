'use client';

import {
  Table,
  TableBody,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { LiveOddsRow } from '@/components/live-odds-row';
import type { MockMatch } from '@/lib/mock-socket';

interface LiveOddsTableProps {
  matches: MockMatch[];
}

// La tabla wrapper solo re-renderiza cuando la lista de matches cambia
// las actualizaciones de odds individuales son manejadas por liveoddsrow via Zustand
export function LiveOddsTable({ matches }: LiveOddsTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-[200px]">Match</TableHead>
          <TableHead className="w-[100px]">Status</TableHead>
          <TableHead className="w-[100px] text-right">Odds</TableHead>
          <TableHead className="w-[100px] text-right">Action</TableHead>
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