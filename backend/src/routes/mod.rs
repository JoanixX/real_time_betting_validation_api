use crate::handlers::{health_check, validate_bet};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health_check", web::get().to(health_check));
    cfg.route("/bets", web::post().to(validate_bet));
}
