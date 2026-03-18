// Se creó un adaptador secundario con implementación postgres 
// del puerto de apuestas

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::{Bet, BetId, MatchId, UserId, Money, Odds, DomainError};
use crate::domain::ports::BetRepository;

pub struct PostgresBetRepository {
    pool: PgPool,
}

impl PostgresBetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Convertidor que antes estaba centralizado en el dominio (ahora está en infraestructura)
fn map_sqlx_error(e: sqlx::Error) -> DomainError {
    match e {
        sqlx::Error::RowNotFound => DomainError::NotFound,
        sqlx::Error::Database(ref db_err) => {
            // se usa el código 23505, que es para una unique_violation en postgres
            if db_err.code().map_or(false, |c| c == "23505") {
                DomainError::Duplicate(db_err.message().to_string())
            } else {
                DomainError::Internal(e.to_string())
            }
        }
        _ => DomainError::Internal(e.to_string()),
    }
}

#[async_trait]
impl BetRepository for PostgresBetRepository {
    async fn save(
        &self,
        bet: &Bet,
    ) -> Result<(), DomainError> {
        let status_str = bet.status.as_str();
        let selection_str = bet.selection.as_str();

        // en la bd actual, amount y odds se almacenan como f64 y
        // en un escenario real ideal se enviarian enteros.
        let amount_f64 = bet.amount.to_decimal();
        let odds_f64 = bet.locked_odds.to_decimal();

        sqlx::query(
            r#"
            INSERT INTO bets (id, user_id, match_id, selection, amount, odds, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(bet.id.0)
        .bind(bet.user_id.0)
        .bind(bet.match_id.0)
        .bind(selection_str)
        .bind(amount_f64)
        .bind(odds_f64)
        .bind(status_str)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(&self, id: BetId) -> Result<Option<Bet>, DomainError> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"SELECT id, user_id, match_id, selection, amount, odds, status FROM bets WHERE id = $1"#
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if let Some(r) = row {
            // Se mapean los numeric de postgres al pattern money y odds
            // en un entorno sin compile-time query type-safe los numeric 
            // pueden venir como decimal u otro tipo.
            // Extraer a string es la forma rudimentaria segura de mapear el numeric,
            // y despues se refactorizaría con tipos decimal dedicados en infraestructura
            
            // sqlx infiere tipos en el fetch
            let amount_str: Option<String> = r.try_get("amount").unwrap_or(None);
            let odds_str: Option<String> = r.try_get("odds").unwrap_or(None);

            let amount = amount_str.unwrap_or_else(|| "0.0".to_string()).parse::<f64>().unwrap_or(0.0);
            let odds = odds_str.unwrap_or_else(|| "0.0".to_string()).parse::<f64>().unwrap_or(0.0);
            
            let id_uuid: Uuid = r.try_get("id").unwrap();
            let user_uuid: Uuid = r.try_get("user_id").unwrap();
            let match_uuid: Uuid = r.try_get("match_id").unwrap();
            let status_str: Option<String> = r.try_get("status").unwrap_or(None);
            let selection_str: String = r.try_get("selection").unwrap_or_else(|_| "HomeWin".to_string());

            let selection = match selection_str.as_str() {
                "HomeWin" => crate::domain::BetSelection::HomeWin,
                "AwayWin" => crate::domain::BetSelection::AwayWin,
                "Draw" => crate::domain::BetSelection::Draw,
                _ => crate::domain::BetSelection::HomeWin,
            };

            let mut bet = Bet::new(
                BetId::from(id_uuid),
                UserId::from(user_uuid),
                MatchId::from(match_uuid),
                selection,
                Money::from_decimal(amount),
                Odds::from_decimal(odds),
            );

            match status_str.as_deref() {
                Some("ACCEPTED") => bet.accept(),
                Some("REJECTED") => bet.reject(),
                _ => {}, // se mantiene en pending
            }

            Ok(Some(bet))
        } else {
            Ok(None)
        }
    }
}