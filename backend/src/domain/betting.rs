// lógica pura de validación de apuestas
// esto encapsula todas las reglas de negocio del dominio

use super::errors::DomainError;
use super::models::{Bet, MatchStatus, SportMatch};
use super::money::Money;

pub trait BetValidationPolicy: Send + Sync {
    /// aqui se valida si una apuesta puede ser aceptada de
    // acuerdo a las reglas del dominio
    fn validate(
        &self,
        bet: &Bet,
        match_info: &SportMatch,
        user_balance: &Money,
    ) -> Result<(), DomainError>;
}

/// implementación estandar de las reglas de negocio
pub struct StandardBetValidationPolicy;

impl StandardBetValidationPolicy {
    pub fn new() -> Self {
        Self
    }

    // el partido debe estar activo
    fn check_match_active(&self, sport_match: &SportMatch) -> Result<(), DomainError> {
        if sport_match.status != MatchStatus::InPlay {
            return Err(DomainError::MatchNotActive {
                match_id: sport_match.id,
                status: sport_match.status.clone(),
            });
        }
        Ok(())
    }

    // el usuario debe tener saldo suficiente
    fn check_sufficient_funds(
        &self,
        bet_amount: &Money,
        user_balance: &Money,
    ) -> Result<(), DomainError> {
        if !bet_amount.is_positive() {
            return Err(DomainError::InvalidAmount(
                "el monto de la apuesta debe ser mayor a cero".to_string(),
            ));
        }

        if bet_amount > user_balance {
            return Err(DomainError::InsufficientFunds {
                available: *user_balance,
                required: *bet_amount,
            });
        }
        Ok(())
    }

    // las odds solicitadas deben coincidir exactamente con las actuales del partido
    fn check_odds_match(
        &self,
        requested_odds: &super::models::Odds,
        current_odds: &super::models::Odds,
    ) -> Result<(), DomainError> {
        if requested_odds != current_odds {
            return Err(DomainError::OddsChanged {
                requested: *requested_odds,
                current: *current_odds,
            });
        }
        Ok(())
    }
}

impl Default for StandardBetValidationPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl BetValidationPolicy for StandardBetValidationPolicy {
    fn validate(
        &self,
        bet: &Bet,
        match_info: &SportMatch,
        user_balance: &Money,
    ) -> Result<(), DomainError> {
        // ejecutar las validaciones en orden de precedencia

        // 1. validar que el monto sea valido y haya fondos
        self.check_sufficient_funds(&bet.amount, user_balance)?;

        // 2. validar que el partido este aceptando apuestas
        self.check_match_active(match_info)?;

        // 3. validar que las odds no hayan cambiado (volatilidad)
        self.check_odds_match(&bet.locked_odds, &match_info.current_odds)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{BetId, MatchId, Odds, UserId};
    use uuid::Uuid;

    fn setup_valid_bet_scenario() -> (Bet, SportMatch, Money) {
        let match_id = MatchId::from(Uuid::new_v4());
        let current_odds = Odds::new(2500); // 2.50

        let sport_match = SportMatch {
            id: match_id,
            status: MatchStatus::InPlay,
            current_odds,
        };

        let bet_amount = Money::new(1000); // 10.00
        let user_balance = Money::new(5000); // 50.00

        let bet = Bet::new(
            BetId::from(Uuid::new_v4()),
            UserId::from(Uuid::new_v4()),
            match_id,
            bet_amount,
            current_odds, // coincide con las actuales
        );

        (bet, sport_match, user_balance)
    }

    #[test]
    fn test_valid_bet_passes_all_checks() {
        let policy = StandardBetValidationPolicy::new();
        let (bet, match_info, balance) = setup_valid_bet_scenario();

        assert!(policy.validate(&bet, &match_info, &balance).is_ok());
    }

    #[test]
    fn test_invalid_amount_is_rejected() {
        let policy = StandardBetValidationPolicy::new();
        let (mut bet, match_info, balance) = setup_valid_bet_scenario();

        // Monto cero
        bet.amount = Money::new(0);
        let result = policy.validate(&bet, &match_info, &balance);
        assert!(matches!(result, Err(DomainError::InvalidAmount(_))));

        // Monto negativo
        bet.amount = Money::new(-100);
        let result = policy.validate(&bet, &match_info, &balance);
        assert!(matches!(result, Err(DomainError::InvalidAmount(_))));
    }

    #[test]
    fn test_insufficient_funds_is_rejected() {
        let policy = StandardBetValidationPolicy::new();
        let (bet, match_info, mut balance) = setup_valid_bet_scenario();

        // Saldo menor a la apuesta (apuesta 1000, saldo 500)
        balance = Money::new(500);

        let result = policy.validate(&bet, &match_info, &balance);

        match result {
            Err(DomainError::InsufficientFunds {
                available,
                required,
            }) => {
                assert_eq!(available.amount_cents, 500);
                assert_eq!(required.amount_cents, 1000);
            }
            _ => panic!("se esperaba InsufficientFunds"),
        }
    }

    #[test]
    fn test_internal_match_status_is_rejected() {
        let policy = StandardBetValidationPolicy::new();
        let (bet, mut match_info, balance) = setup_valid_bet_scenario();

        let invalid_statuses = vec![
            MatchStatus::NotStarted,
            MatchStatus::Finished,
            MatchStatus::Suspended,
        ];

        for status in invalid_statuses {
            match_info.status = status;
            let result = policy.validate(&bet, &match_info, &balance);
            assert!(matches!(result, Err(DomainError::MatchNotActive { .. })));
        }
    }

    #[test]
    fn test_odds_changed_is_rejected() {
        let policy = StandardBetValidationPolicy::new();
        let (mut bet, match_info, balance) = setup_valid_bet_scenario();

        // las odds de la apuesta ya no coinciden con las del partido
        bet.locked_odds = Odds::new(2600); // 2.60 != 2.50

        let result = policy.validate(&bet, &match_info, &balance);

        match result {
            Err(DomainError::OddsChanged { requested, current }) => {
                assert_eq!(requested.value_thousandths, 2600);
                assert_eq!(current.value_thousandths, 2500);
            }
            _ => panic!("se esperaba OddsChanged"),
        }
    }
}