use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
pub struct BetTicket {
    pub user_id: Uuid,
    pub match_id: Uuid,
    pub amount: f64,
    pub odds: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BetStatus {
    Pending,
    Validated,
    Rejected,
}
