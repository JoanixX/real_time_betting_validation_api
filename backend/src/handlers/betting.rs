use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::application::PlaceBetUseCase;
use crate::telemetry::metrics::{BETTING_API_BETS_PLACED_TOTAL, BETTING_API_BETS_REJECTED_TOTAL};
use crate::domain::{Bet, BetId, MatchId, UserId, Money, Odds};
use super::dto::{ValidateBetRequest, PlaceBetResponse};

#[tracing::instrument(
    name = "Validando una nueva apuesta",
    skip(item, use_case),
    fields(
        user_id = %item.user_id,
        match_id = %item.match_id
    )
)]
pub async fn validate_bet(
    item: web::Json<ValidateBetRequest>,
    use_case: web::Data<PlaceBetUseCase>,
) -> HttpResponse {
    // traducir dto primitivo a una entidad de dominio rica
    let bet_id = BetId::from(Uuid::new_v4());
    let user_id = UserId::from(item.user_id);
    let match_id = MatchId::from(item.match_id);
    
    // parseamos el selection a enum
    let selection = match item.selection.as_str() {
        "HomeWin" => crate::domain::BetSelection::HomeWin,
        "AwayWin" => crate::domain::BetSelection::AwayWin,
        "Draw" => crate::domain::BetSelection::Draw,
        _ => return HttpResponse::BadRequest().json("Selección inválida"),
    };

    // Convertir de dto a tipos internos de dominio
    let amount = Money::from_decimal(item.amount);
    let odds = Odds::from_decimal(item.odds);

    let bet = Bet::new(
        bet_id,
        user_id,
        match_id,
        selection,
        amount,
        odds,
    );

    // Se manda al caso de uso
    match use_case.execute(bet).await {
        Ok(result) => {
            // esto registra la metrica que confirma que todo god
            BETTING_API_BETS_PLACED_TOTAL.inc();
            
            // se traduce la entidad rica a un dto simple
            HttpResponse::Created().json(PlaceBetResponse {
                bet_id: result.bet.id.0,
                user_id: result.bet.user_id.0,
                match_id: result.bet.match_id.0,
                selection: result.bet.selection.as_str().to_string(),
                amount: result.bet.amount.to_decimal(),
                odds: result.bet.locked_odds.to_decimal(),
                status: result.bet.status.as_str().to_string(),
            })
        }
        Err(e) => {
            // registramos aqui la metrica de rechazo
            // o sea si una regla de negocio se rompe
            BETTING_API_BETS_REJECTED_TOTAL.inc();
            crate::errors::domain_error_to_response(e)
        }
    }
}