// traits que definen las interfaces, el dominio y los casos
// de uso dependen de los traits y las implementaciones
// concretas van en la carpeta infrastructure

use async_trait::async_trait;
use uuid::Uuid;

use super::errors::DomainError;
use super::models::{Bet, BetId, BetStatus, SportMatch, MatchId, UserId};

// Puerto de apuestas
#[async_trait]
pub trait BetRepository: Send + Sync {
    async fn save(&self, bet: &Bet) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: BetId) -> Result<Option<Bet>, DomainError>;
}

// Puerto de partidos y cuotas (nuevo para soportar la validación)
#[async_trait]
pub trait MatchRepository: Send + Sync {
    async fn find_by_id(&self, id: MatchId) -> Result<Option<SportMatch>, DomainError>;
}

// Puerto de usuarios
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(
        &self,
        id: UserId,
        email: &str,
        password_hash: &str,
        name: &str,
    ) -> Result<(), DomainError>;

    async fn find_by_email(&self, email: &str) -> Result<Option<UserRecord>, DomainError>;
    // se necesita para la validacion financiera
    async fn get_balance(&self, id: UserId) -> Result<crate::domain::Money, DomainError>;
}

// registro devuelto por el repositorio con hash
#[derive(Debug)]
pub struct UserRecord {
    pub id: Uuid,
    pub password_hash: String,
    pub name: Option<String>,
}

// Puerto de cache
#[async_trait]
pub trait CachePort: Send + Sync {
    async fn set(&self, key: &str, value: &str, expire_secs: usize) -> Result<(), DomainError>;
    async fn get(&self, key: &str) -> Result<Option<String>, DomainError>;
}

// Puerto de hashing de contraseñas
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, DomainError>;
    fn verify(&self, password: &str, hash: &str) -> Result<bool, DomainError>;
}

// Puerto de estado de apuestas de alta velocidad (Redis)
#[async_trait]
pub trait BettingStateRepository: Send + Sync {
    // mete la apuesta y verifica atómicamente que el saldo sea mayor o igual al amount
    // y que las expected_odds coincidan con las actuales en memoria.
    async fn place_bet_atomically(
        &self,
        bet_id: BetId,
        user_id: UserId,
        match_id: MatchId,
        selection: crate::domain::BetSelection,
        amount: crate::domain::Money,
        expected_odds: crate::domain::Odds,
    ) -> Result<(), DomainError>;
}