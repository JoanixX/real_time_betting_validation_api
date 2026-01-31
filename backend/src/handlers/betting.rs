use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::models::BetTicket;

#[derive(serde::Deserialize)]
pub struct ValidateBetRequest {
    pub user_id: Uuid,
    pub match_id: Uuid,
    pub amount: f64,
    pub odds: f64,
}

#[tracing::instrument(
    name = "Validating a new bet",
    skip(item, pool),
    fields(
        user_id = %item.user_id,
        match_id = %item.match_id
    )
)]
pub async fn validate_bet(
    item: web::Json<ValidateBetRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let ticket = BetTicket {
        user_id: item.user_id,
        match_id: item.match_id,
        amount: item.amount,
        odds: item.odds,
    };

    // Insert into DB
    let bet_id = Uuid::new_v4();
    let status_str = "VALIDATED"; 

    match sqlx::query!(
        r#"
        INSERT INTO bets (id, user_id, match_id, amount, odds, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        bet_id,
        ticket.user_id,
        ticket.match_id,
        ticket.amount,
        ticket.odds,
        status_str,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info!("Bet validated and stored successfully");
            HttpResponse::Ok().json(ticket)
        },
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
