// errores de dominio puros
// los errores de infraestructura (como sqlx) se manejan en los adaptadores

use thiserror::Error;
use super::models::{MatchId, MatchStatus, Odds};
use super::money::Money;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("error de validación: {0}")]
    Validation(String),

    #[error("entidad no encontrada")]
    NotFound,

    #[error("credenciales inválidas")]
    AuthenticationFailed,

    #[error("entidad duplicada: {0}")]
    Duplicate(String),

    #[error("error interno: {0}")]
    Internal(String),

    // Nuevos errores de negocio específicos
    #[error("saldo insuficiente. disponible: {available:?}, requerido: {required:?}")]
    InsufficientFunds { available: Money, required: Money },

    #[error("el partido no está activo. estado actual: {status:?}")]
    MatchNotActive { match_id: MatchId, status: MatchStatus },

    #[error("las cuotas han cambiado. solicitadas: {requested:?}, actuales: {current:?}")]
    OddsChanged { requested: Odds, current: Odds },

    #[error("monto de apuesta inválido: {0}")]
    InvalidAmount(String),
}