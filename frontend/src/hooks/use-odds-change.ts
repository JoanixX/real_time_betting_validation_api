'use client';

import { useRef, useEffect, useState } from 'react';

// detecta si un valor numérico subió o bajó vs el render anterior
// retorna estados y un key incremental para la animación
export function useOddsChange(currentValue: number | undefined) {
  const prevRef = useRef<number | undefined>(undefined);
  const [direction, setDirection] = useState<'up' | 'down' | null>(null);
  const [flashKey, setFlashKey] = useState(0);

  useEffect(() => {
    if (currentValue === undefined || prevRef.current === undefined) {
      prevRef.current = currentValue;
      return;
    }

    if (currentValue > prevRef.current) {
      setDirection('up');
      setFlashKey((k) => k + 1);
    } else if (currentValue < prevRef.current) {
      setDirection('down');
      setFlashKey((k) => k + 1);
    }

    prevRef.current = currentValue;
  }, [currentValue]);

  // se limpia la dirección después de que la animación termine (600ms)
  useEffect(() => {
    if (direction === null) return;
    const timer = setTimeout(() => setDirection(null), 600);
    return () => clearTimeout(timer);
  }, [direction, flashKey]);

  return { direction, flashKey };
}