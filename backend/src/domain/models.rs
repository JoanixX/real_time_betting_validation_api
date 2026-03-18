// entidades de dominio puras sin dtos de http
// los request/response types van en el adaptador de handlers

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fmt::Display;

use super::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchId(pub Uuid);

impl Display for MatchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for MatchId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BetId(pub Uuid);

impl Display for BetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for BetId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Representa las cuotas (odds) usando un entero en milésimas
/// por ejemplo : 2.50 = 2500, 1.05 = 1050
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Odds {
    pub value_thousandths: u32,
}

impl Odds {
    pub fn new(value_thousandths: u32) -> Self {
        Self { value_thousandths }
    }

    /// convierte un float a milesimas solo para la entrada de datos
    pub fn from_decimal(odds: f64) -> Self {
        Self {
            value_thousandths: (odds * 1000.0).round() as u32,
        }
    }

    pub fn to_decimal(&self) -> f64 {
        self.value_thousandths as f64 / 1000.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchStatus {
    NotStarted,
    InPlay,
    Finished,
    Suspended,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BetSelection {
    HomeWin,
    AwayWin,
    Draw,
}

impl BetSelection {
    pub fn as_str(&self) -> &'static str {
        match self {
            BetSelection::HomeWin => "HomeWin",
            BetSelection::AwayWin => "AwayWin",
            BetSelection::Draw => "Draw",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SportMatch {
    pub id: MatchId,
    pub status: MatchStatus,
    pub current_odds: Odds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BetStatus {
    Pending,
    Accepted,
    Rejected,
    Won,
    Lost,
}

impl BetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BetStatus::Pending => "PENDING",
            BetStatus::Accepted => "ACCEPTED",
            BetStatus::Rejected => "REJECTED",
            BetStatus::Won => "WON",
            BetStatus::Lost => "LOST",
        }
    }
}

// representa a un usuario dentro del subdominio de apuestas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
    pub balance: Money,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bet {
    pub id: BetId,
    pub user_id: UserId,
    pub match_id: MatchId,
    pub selection: BetSelection,
    pub amount: Money,
    pub locked_odds: Odds,
    pub status: BetStatus,
}

impl Bet {
    pub fn new(
        id: BetId,
        user_id: UserId,
        match_id: MatchId,
        selection: BetSelection,
        amount: Money,
        locked_odds: Odds,
    ) -> Self {
        Self {
            id,
            user_id,
            match_id,
            selection,
            amount,
            locked_odds,
            status: BetStatus::Pending,
        }
    }

    pub fn accept(&mut self) {
        self.status = BetStatus::Accepted;
    }

    pub fn reject(&mut self) {
        self.status = BetStatus::Rejected;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newtypes_conversion() {
        let uuid = Uuid::new_v4();
        let user_id = UserId::from(uuid);
        let match_id = MatchId::from(uuid);

        assert_eq!(user_id.0, uuid);
        assert_eq!(match_id.0, uuid);
        
        // esto no compilaria por los newtypes, lo cual es el objetivo
        // assert_eq!(user_id, match_id)
    }

    #[test]
    fn test_odds_conversion() {
        let odds = Odds::from_decimal(2.50);
        assert_eq!(odds.value_thousandths, 2500);
        assert_eq!(odds.to_decimal(), 2.5);

        let odds2 = Odds::from_decimal(1.055); // 1.055 * 1000 = 1055
        assert_eq!(odds2.value_thousandths, 1055);
        assert_eq!(odds2.to_decimal(), 1.055);
    }

    #[test]
    fn test_odds_comparison() {
        let odds1 = Odds::new(2500);
        let odds2 = Odds::new(2500);
        let odds3 = Odds::new(1500);

        assert_eq!(odds1, odds2);
        assert!(odds1 > odds3);
    }

    #[test]
    fn test_bet_creation_and_status() {
        let mut bet = Bet::new(
            BetId::from(Uuid::new_v4()),
            UserId::from(Uuid::new_v4()),
            MatchId::from(Uuid::new_v4()),
            BetSelection::HomeWin,
            Money::new(1000),
            Odds::new(2000),
        );

        assert_eq!(bet.status, BetStatus::Pending);

        bet.accept();
        assert_eq!(bet.status, BetStatus::Accepted);
        assert_eq!(bet.status.as_str(), "ACCEPTED");

        bet.reject();
        assert_eq!(bet.status, BetStatus::Rejected);
        assert_eq!(bet.status.as_str(), "REJECTED");
    }
}