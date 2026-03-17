'use client';

import { useEffect, useState } from 'react';
import { LiveOddsTable } from '@/components/live-odds-table';
import { BettingSlip } from '@/components/betting-slip';
import { ConnectionStatus } from '@/components/connection-status';
import { Badge } from '@/components/ui/badge';
import { startMockOddsStream, getMockMatches, type MockMatch } from '@/lib/mock-socket';
import { useBettingStore } from '@/store/betting-store';
import { Activity, Zap } from 'lucide-react';
import { MATCH_STATUS, BET_STATUS } from '@/lib/constants';

export default function DashboardPage() {
  const [matches, setMatches] = useState<MockMatch[]>([]);
  const activityLog = useBettingStore((s) => s.activityLog);

  // iniciar mock stream al montar el dashboard
  useEffect(() => {
    setMatches(getMockMatches());
    const cleanup = startMockOddsStream();
    return cleanup;
  }, []);

  return (
    <div className="space-y-6">
      {/* header con estado de conexión */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Dashboard en Vivo</h1>
          <p className="text-sm text-muted-foreground">
            Cuotas en tiempo real desde el backend de alta concurrencia en Rust
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Badge variant="outline" className="gap-1">
            <Zap className="h-3 w-3" />
            {matches.filter((m) => m.status === MATCH_STATUS.IN_PLAY).length} en vivo
          </Badge>
          <ConnectionStatus />
        </div>
      </div>

      {/* layout principal: tabla + betting slip */}
      <div className="grid gap-6 lg:grid-cols-3">
        <div className="lg:col-span-2">
          <LiveOddsTable matches={matches} />
        </div>
        <div className="lg:col-span-1">
          <BettingSlip />
        </div>
      </div>

      {/* Log de actividad */}
      {activityLog.length > 0 && (
        <div className="rounded-lg border p-4">
          <h2 className="mb-3 flex items-center gap-2 text-sm font-semibold">
            <Activity className="h-4 w-4" />
            Actividad Reciente
          </h2>
          <div className="space-y-1">
            {activityLog.slice(0, 5).map((entry, i) => (
              <div
                key={`${entry.timestamp}-${i}`}
                className="flex items-center justify-between text-xs text-muted-foreground"
              >
                <span>
                  ${entry.amount.toFixed(2)} apuesta
                  {entry.status && (
                    <Badge
                      variant={entry.status === BET_STATUS.ACCEPTED ? 'default' : 'destructive'}
                      className="ml-2 text-[10px]"
                    >
                      {entry.status}
                    </Badge>
                  )}
                </span>
                <span className="tabular-nums">{entry.latency_ms.toFixed(0)}ms</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}