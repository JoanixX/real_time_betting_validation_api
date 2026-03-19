// errores de dominio puros
// los errores de infraestructura (como sqlx) se manejan en los adaptadores

use super::models::{MatchId, MatchStatus, Odds};
use super::money::Money;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Error de validación: {0}")]
    Validation(String),

    #[error("Entidad no encontrada")]
    NotFound,

    #[error("Credenciales inválidas")]
    AuthenticationFailed,

    #[error("Entidad duplicada: {0}")]
    Duplicate(String),

    #[error("Error interno: {0}")]
    Internal(String),

    // Nuevos errores de negocio especificos
    #[error("Saldo insuficiente. Disponible: {available:?}, Requerido: {required:?}")]
    InsufficientFunds { available: Money, required: Money },

    #[error("El partido no está activo. Estado actual: {status:?}")]
    MatchNotActive {
        match_id: MatchId,
        status: MatchStatus,
    },

    #[error("Las cuotas han cambiado. Solicitadas: {requested:?}, Actuales: {current:?}")]
    OddsChanged { requested: Odds, current: Odds },

    #[error("Monto de apuesta inválido: {0}")]
    InvalidAmount(String),

    #[error("Falla de infraestructura: {0}")]
    InfrastructureError(String),
}