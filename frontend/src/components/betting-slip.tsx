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
  const selections = useSelectionsStore((s) => s.selections);
  const removeSelection = useSelectionsStore((s) => s.removeSelection);
  const updateAmount = useSelectionsStore((s) => s.updateAmount);
  const clearAll = useSelectionsStore((s) => s.clearAll);
  const userId = useAuthStore((s) => s.user_id);

  const { mutate: placeBet, isPending } = usePlaceBet();

  // convertir Map a array para renderizar
  const selectionsList = useMemo(
    () => Array.from(selections.values()),
    [selections]
  );

  // total apostado
  const totalAmount = useMemo(
    () => selectionsList.reduce((sum, s) => sum + s.amount, 0),
    [selectionsList]
  );

  // ganancia potencial
  const potentialReturn = useMemo(
    () =>
      selectionsList.reduce((sum, s) => sum + s.amount * s.odds, 0),
    [selectionsList]
  );

  const handleAmountChange = useCallback(
    (matchId: string, value: string) => {
      const num = parseFloat(value);
      if (!isNaN(num) && num >= 0) {
        updateAmount(matchId, num);
      }
    },
    [updateAmount]
  );

  const handleSubmitAll = useCallback(() => {
    const betUserId = userId ?? 'demo-user';
    for (const selection of selectionsList) {
      placeBet({
        user_id: betUserId,
        match_id: selection.matchId,
        amount: selection.amount,
        odds: selection.odds,
      });
    }
    clearAll();
  }, [userId, selectionsList, placeBet, clearAll]);

  return (
    <Card className="flex h-full flex-col">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-lg">Boleta de apuestas</CardTitle>
            <CardDescription>
              {selectionsList.length === 0
                ? 'Selecciona partidos para apostar'
                : `${selectionsList.length} selección${selectionsList.length > 1 ? 'es' : ''}`}
            </CardDescription>
          </div>
          {selectionsList.length > 0 && (
            <Button
              variant="ghost"
              size="sm"
              onClick={clearAll}
              className="text-muted-foreground"
            >
              Clear all
            </Button>
          )}
        </div>
      </CardHeader>

      <CardContent className="flex-1 space-y-3 overflow-y-auto">
        {selectionsList.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-center text-muted-foreground">
            <AlertCircle className="mb-2 h-8 w-8" />
            <p className="text-sm">
              Click &ldquo;Apostar&rdquo; en un partido para agregarlo a tu boleta
            </p>
          </div>
        ) : (
          selectionsList.map((selection) => (
            <div
              key={selection.matchId}
              className="rounded-lg border bg-muted/30 p-3"
            >
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
                  onClick={() => removeSelection(selection.matchId)}
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
                  onChange={(e) =>
                    handleAmountChange(selection.matchId, e.target.value)
                  }
                  className="h-8 text-sm"
                />
                <span className="whitespace-nowrap text-xs text-muted-foreground">
                  → ${(selection.amount * selection.odds).toFixed(2)}
                </span>
              </div>
            </div>
          ))
        )}
      </CardContent>

      {selectionsList.length > 0 && (
        <CardFooter className="flex-col space-y-3 border-t pt-4">
          <div className="flex w-full justify-between text-sm">
            <span className="text-muted-foreground">Total apostado</span>
            <span className="font-medium">${totalAmount.toFixed(2)}</span>
          </div>
          <div className="flex w-full justify-between text-sm">
            <span className="text-muted-foreground">Ganancia potencial</span>
            <span className="font-bold text-green-500">
              ${potentialReturn.toFixed(2)}
            </span>
          </div>
          <Button
            className="w-full"
            size="lg"
            onClick={handleSubmitAll}
            disabled={isPending || totalAmount === 0}
          >
            {isPending ? (
              <span className="animate-pulse">Colocando apuestas...</span>
            ) : (
              <>
                <Send className="mr-2 h-4 w-4" />
                Colocar {selectionsList.length} apuesta{selectionsList.length > 1 ? 's' : ''}
              </>
            )}
          </Button>
        </CardFooter>
      )}
    </Card>
  );
}