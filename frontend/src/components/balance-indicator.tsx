'use client';

import React, { useEffect, useRef } from 'react';
import { useBalanceStore } from '@/store/balance-store';
import { Wallet } from 'lucide-react';

// formateador de moneda en dolares instanciado una sola vez fuera del render
const currencyFormatter = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
  minimumFractionDigits: 2,
});

// convierte centavos enteros a string formateado
function formatCents(cents: number): string {
  return currencyFormatter.format(cents / 100);
}

const BalanceIndicator = React.memo(function BalanceIndicator() {
  const balance = useBalanceStore((s) => s.balance);
  const containerRef = useRef<HTMLDivElement>(null);
  const amountRef = useRef<HTMLSpanElement>(null);
  const prevBalanceRef = useRef<number>(balance);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  // efecto flash O(1): manipula classList directamente sin causar re renderizaciones
  useEffect(() => {
    if (prevBalanceRef.current === balance) return;

    const isUp = balance > prevBalanceRef.current;

    // limpiamos timeout previo si hay actualizaciones rapidas
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    containerRef.current?.classList.remove('bg-green-500/20', 'bg-red-500/20');
    amountRef.current?.classList.remove('text-green-500', 'text-red-500');
    const bgClass = isUp ? 'bg-green-500/20' : 'bg-red-500/20';
    const textClass = isUp ? 'text-green-500' : 'text-red-500';
    containerRef.current?.classList.add(bgClass, 'transition-colors', 'duration-75');
    amountRef.current?.classList.add(textClass, 'transition-colors', 'duration-75');

    // removemos el flash despues de 500ms
    timeoutRef.current = setTimeout(() => {
      containerRef.current?.classList.remove(bgClass);
      amountRef.current?.classList.remove(textClass);
    }, 500);

    prevBalanceRef.current = balance;

    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, [balance]);

  return (
    <div
      ref={containerRef}
      className="flex items-center gap-2 rounded-lg border border-border/50 px-3 py-1.5"
    >
      <Wallet className="h-4 w-4 text-muted-foreground" />
      <span
        ref={amountRef}
        className="text-sm font-semibold tabular-nums"
      >
        {formatCents(balance)}
      </span>
    </div>
  );
});

export { BalanceIndicator };