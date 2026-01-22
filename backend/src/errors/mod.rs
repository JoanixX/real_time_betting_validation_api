use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Error Interno del Servidor")]
    UnexpectedError(#[from] anyhow::Error),
    
    #[error("Error de Validación: {0}")]
    ValidationError(String),
    
    #[error("Recurso No Encontrado")]
    NotFoundError,
    
    #[error("Error de Base de Datos")]
    DatabaseError(#[from] sqlx::Error),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFoundError => StatusCode::NOT_FOUND,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        // Podríamos personalizar el formato de respuesta aquí (ej. JSON estandarizado)
        match self {
            AppError::UnexpectedError(_) => {
                // No queremos filtrar detalles internos al cliente en errores 500
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Error Interno del Servidor"
                }))
            }
            AppError::ValidationError(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Error de Validación",
                    "message": msg
                }))
            }
            AppError::NotFoundError => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "No Encontrado"
                }))
            }
            AppError::DatabaseError(_) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Error Interno del Servidor"
                }))
            }
        }
    }
}
