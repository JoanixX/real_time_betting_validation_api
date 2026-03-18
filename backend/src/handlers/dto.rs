// dtos de los adaptadores primarios http
// los tipos creados en esta capa pertenecen a la capa de handlers, no al dominio

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Request para validar o colocar una apuesta
#[derive(Debug, Deserialize)]
pub struct ValidateBetRequest {
    pub user_id: Uuid,
    pub match_id: Uuid,
    pub selection: String,
    pub amount: f64,
    pub odds: f64,
}

// Respuesta de apuesta colocada
#[derive(Debug, Serialize)]
pub struct PlaceBetResponse {
    pub bet_id: Uuid,
    pub user_id: Uuid,
    pub match_id: Uuid,
    pub selection: String,
    pub amount: f64,
    pub odds: f64,
    pub status: String,
}

// Request de registro
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

// Request de login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}