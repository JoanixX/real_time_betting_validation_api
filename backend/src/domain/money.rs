// patrón money para representar montos exactos y evitar problemas
// de precisión con f64. almacena el monto internamente en centavos.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money {
    pub amount_cents: i64,
}

impl Money {
    pub fn new(amount_cents: i64) -> Self {
        Self { amount_cents }
    }

    // convierte un f64 a Money de forma segura
    pub fn from_decimal(amount: f64) -> Self {
        Self {
            amount_cents: (amount * 100.0).round() as i64,
        }
    }

    // convierte Money a f64 (solo para presentation/dto)
    pub fn to_decimal(&self) -> f64 {
        self.amount_cents as f64 / 100.0
    }

    pub fn is_positive(&self) -> bool {
        self.amount_cents > 0
    }

    pub fn is_zero(&self) -> bool {
        self.amount_cents == 0
    }

    pub fn zero() -> Self {
        Self { amount_cents: 0 }
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            amount_cents: self.amount_cents + other.amount_cents,
        }
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            amount_cents: self.amount_cents - other.amount_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_creation() {
        let m = Money::new(100);
        assert_eq!(m.amount_cents, 100);
    }

    #[test]
    fn test_money_from_decimal() {
        let m = Money::from_decimal(10.50);
        assert_eq!(m.amount_cents, 1050);

        let m2 = Money::from_decimal(10.509); // redondea correctamente
        assert_eq!(m2.amount_cents, 1051);
    }

    #[test]
    fn test_money_to_decimal() {
        let m = Money::new(1050);
        assert_eq!(m.to_decimal(), 10.5);
    }

    #[test]
    fn test_money_addition() {
        let m1 = Money::new(100);
        let m2 = Money::new(200);
        assert_eq!(m1 + m2, Money::new(300));
    }

    #[test]
    fn test_money_subtraction() {
        let m1 = Money::new(200);
        let m2 = Money::new(100);
        assert_eq!(m1 - m2, Money::new(100));

        // Permite negativos para lógica contable temporal
        let m3 = Money::new(50);
        assert_eq!(m3 - m2, Money::new(-50));
    }

    #[test]
    fn test_money_comparison() {
        let m1 = Money::new(100);
        let m2 = Money::new(200);
        let m3 = Money::new(100);

        assert!(m1 < m2);
        assert!(m2 > m1);
        assert_eq!(m1, m3);
    }

    #[test]
    fn test_money_checks() {
        assert!(Money::new(10).is_positive());
        assert!(!Money::new(-10).is_positive());
        assert!(!Money::new(0).is_positive());
        assert!(Money::new(0).is_zero());
        assert_eq!(Money::zero(), Money::new(0));
    }
}