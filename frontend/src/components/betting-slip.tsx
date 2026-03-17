'use client';

import { useCallback, useMemo } from 'react';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { useSelectionsStore } from '@/store/selections-store';
import { useAuthStore } from '@/store/auth-store';
import { usePlaceBet } from '@/hooks/use-place-bet';
import { Trash2, Send, AlertCircle } from 'lucide-react';

export function BettingSlip() {
  const selection = useSelectionsStore((s) => s.selection);
  const clearSelection = useSelectionsStore((s) => s.clearSelection);
  const updateAmount = useSelectionsStore((s) => s.updateAmount);
  const userId = useAuthStore((s) => s.user_id);

  const { mutate: placeBet, isPending } = usePlaceBet();

  const handleSubmit = useCallback(() => {
    if (!selection) return;
    const betUserId = userId ?? 'demo-user';
    placeBet({
      user_id: betUserId,
      match_id: selection.matchId,
      amount: selection.amount,
      odds: selection.odds,
    });
    // ahora limpiamos luego de hacer el intento
    clearSelection();
  }, [userId, selection, placeBet, clearSelection]);

  const handleAmountChange = useCallback(
    (value: string) => {
      const num = parseFloat(value);
      if (!isNaN(num) && num >= 0) {
        updateAmount(num);
      }
    },
    [updateAmount]
  );

  return (
    <Card className="flex h-full flex-col">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-lg">Boleta de apuestas</CardTitle>
            <CardDescription>
              {!selection
                ? 'Selecciona una cuota para apostar'
                : '1 selección rápida en curso'}
            </CardDescription>
          </div>
          {selection && (
            <Button
              variant="ghost"
              size="sm"
              onClick={clearSelection}
              className="text-muted-foreground"
            >
              Cerrar
            </Button>
          )}
        </div>
      </CardHeader>

      <CardContent className="flex-1 space-y-3 overflow-y-auto">
        {!selection ? (
          <div className="flex flex-col items-center justify-center py-8 text-center text-muted-foreground">
            <AlertCircle className="mb-2 h-8 w-8" />
            <p className="text-sm">
              Click &ldquo;Apostar&rdquo; en un partido para armar tu slip rápido
            </p>
          </div>
        ) : (
          <div className="rounded-lg border bg-muted/30 p-3">
            <div className="mb-2 flex items-start justify-between">
              <div className="flex-1">
                <p className="text-sm font-medium">
                  {selection.homeTeam} vs {selection.awayTeam}
                </p>
                <Badge variant="outline" className="mt-1">
                  Cuota: {selection.odds.toFixed(2)}
                </Badge>
              </div>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 text-muted-foreground hover:text-destructive"
                onClick={clearSelection}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-muted-foreground">$</span>
              <Input
                type="number"
                min={0}
                step={5}
                value={selection.amount}
                onChange={(e) => handleAmountChange(e.target.value)}
                className="h-8 text-sm"
              />
              <span className="whitespace-nowrap text-xs text-muted-foreground">
                → ${(selection.amount * selection.odds).toFixed(2)}
              </span>
            </div>
          </div>
        )}
      </CardContent>

      {selection && (
        <CardFooter className="flex-col space-y-3 border-t pt-4">
          <div className="flex w-full justify-between text-sm">
            <span className="text-muted-foreground">Monto</span>
            <span className="font-medium">${selection.amount.toFixed(2)}</span>
          </div>
          <div className="flex w-full justify-between text-sm">
            <span className="text-muted-foreground">Ganancia potencial</span>
            <span className="font-bold text-green-500">
              ${(selection.amount * selection.odds).toFixed(2)}
            </span>
          </div>
          <Button
            className="w-full"
            size="lg"
            onClick={handleSubmit}
            disabled={isPending || selection.amount <= 0}
          >
            {isPending ? (
              <span className="animate-pulse">Colocando apuesta...</span>
            ) : (
              <>
                <Send className="mr-2 h-4 w-4" />
                Colocar apuesta
              </>
            )}
          </Button>
        </CardFooter>
      )}
    </Card>
  );
}