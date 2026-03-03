// Este es el mapeo de errores de dominio a respuestas http
// basicamente un puente entre la arquitectura y el protocolo http

use actix_web::HttpResponse;
use crate::domain::DomainError;

// convierte un error de dominio en httpResponse
pub fn domain_error_to_response(error: DomainError) -> HttpResponse {
    match error {
        DomainError::Validation(msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Error de validación",
                "message": msg
            }))
        }
        DomainError::InsufficientFunds { available, required } => {
            HttpResponse::PaymentRequired().json(serde_json::json!({
                "error": "Saldo insuficiente",
                "available_cents": available.amount_cents,
                "required_cents": required.amount_cents
            }))
        }
        DomainError::MatchNotActive { match_id, status } => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": "El partido no está activo para apuestas",
                "match_id": match_id.0.to_string(),
                "current_status": format!("{:?}", status)
            }))
        }
        DomainError::OddsChanged { requested, current } => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": "Las cuotas han cambiado",
                "requested_odds": requested.to_decimal(),
                "current_odds": current.to_decimal()
            }))
        }
        DomainError::InvalidAmount(msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Monto inválido",
                "message": msg
            }))
        }
        DomainError::NotFound => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "No encontrado"
            }))
        }
        DomainError::AuthenticationFailed => {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Credenciales inválidas"
            }))
        }
        DomainError::Duplicate(msg) => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": "Entidad duplicada",
                "message": msg
            }))
        }
        DomainError::Internal(msg) => {
            // logueamos el error real pero no lo exponemos al cliente
            tracing::error!("Error interno: {}", msg);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Error interno del servidor"
            }))
        }
        DomainError::InfrastructureError(msg) => {
            // logueamos el error de la infraestructura redis
            tracing::error!("Error de infraestructura: {}", msg);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Ocurrió un error procesando la transacción"
            }))
        }
    }
}