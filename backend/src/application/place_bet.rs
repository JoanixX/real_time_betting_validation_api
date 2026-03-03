// Colocar apuesta
// orquesta la lógica de negocio usando solo los puertos

use std::sync::Arc;
use crate::domain::{Bet, DomainError, ports::{BettingStateRepository, CachePort}};

pub struct PlaceBetUseCase {
    bet_state_repo: Arc<dyn BettingStateRepository>,
    cache: Arc<dyn CachePort>,
}

// respuesta del caso de uso
#[derive(Debug)]
pub struct PlaceBetResult {
    pub bet: Bet,
}

impl PlaceBetUseCase {
    pub fn new(
        bet_state_repo: Arc<dyn BettingStateRepository>,
        cache: Arc<dyn CachePort>,
    ) -> Self {
        Self { 
            bet_state_repo,
            cache,
        }
    }

    pub async fn execute(&self, mut bet: Bet) -> Result<PlaceBetResult, DomainError> {
        // 1. hacemos la validacion y debito atómicamente del redis
        self.bet_state_repo.place_bet_atomically(
            bet.id,
            bet.user_id,
            bet.match_id,
            bet.amount,
            bet.locked_odds
        ).await?;

        // 2. transicion de estado a Aceptada
        bet.accept();

        tracing::info!(
            bet_id = %bet.id,
            user_id = %bet.user_id,
            "Apuesta validada y empujada a la cola pending atómicamente"
        );

        // 3. cache de ultima apuesta (best-effort)
        let cache_key = format!("last_bet:{}", bet.user_id);
        if let Err(e) = self.cache.set(&cache_key, &bet.id.to_string(), 60).await {
            tracing::warn!("no se pudo actualizar la cache: {:?}", e);
        }

        Ok(PlaceBetResult {
            bet,
        })
    }
}