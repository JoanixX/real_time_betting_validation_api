use super::models::BetTicket;
use std::future::Future;
use std::pin::Pin;

pub trait BetValidator {
    fn validate(&self, ticket: &BetTicket) -> Pin<Box<dyn Future<Output = bool> + Send>>;
}
