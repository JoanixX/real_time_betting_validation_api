use async_trait::async_trait;
use deadpool_redis::{Pool, redis::AsyncCommands};
use redis::Script;
use crate::domain::{UserId, MatchId, Money, Odds, DomainError};
use crate::domain::ports::BettingStateRepository;

pub struct RedisBettingStateRepository {
    pool: Pool,
}

impl RedisBettingStateRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

// helper para mapear errores del pool de redis a nuestro domainError
fn map_redis_error(e: impl std::fmt::Display) -> DomainError {
    DomainError::InfrastructureError(e.to_string())
}

#[async_trait]
impl BettingStateRepository for RedisBettingStateRepository {
    async fn place_bet_atomically(
        &self,
        bet_id: crate::domain::BetId,
        user_id: UserId,
        match_id: MatchId,
        selection: crate::domain::BetSelection,
        amount: Money,
        expected_odds: Odds,
    ) -> Result<(), DomainError> {
        // obtenemos conexion asíncrona dedicada del deadpool
        let mut conn = self.pool.get().await.map_err(map_redis_error)?;

        // estas son las llaves involucradas que el script atómico leera
        let match_odds_key = format!("match:{}:odds", match_id.0);
        let user_balance_key = format!("user:{}:balance", user_id.0);
        let pending_bets_key = "bets_stream".to_string();

        // aqui hacemos algo interesante, usamos lua para no utilizar
        // la lectura y escritura por separado, lo que podria causar race conditions (watch)
        // keys[1] -> match odds
        // keys[2] -> user balance
        // keys[3] -> pending bets stream
        // argv[1] -> expected_odds (en milesimas)
        // argv[2] -> amount (en centavos)
        // argv[3] -> bet id
        // argv[4] -> user id
        // argv[5] -> match id
        // argv[6] -> selection
        
        let script = Script::new(
            r#"
            -- 1. Validar cuotas actuales
            local current_odds = redis.call("GET", KEYS[1])
            if current_odds == false or current_odds ~= ARGV[1] then
                return -2 -- Error code: las cuotas no coinciden o el partido no existe/no tiene cuotas activas
            end

            -- 2. Validar que tenga el saldo disponible
            local balance = redis.call("GET", KEYS[2])
            if balance == false or tonumber(balance) < tonumber(ARGV[2]) then
                return -1 -- Error code: fondos insuficientes
            end

            -- 3. Restar atómicamente el saldo y permitir apuesta
            redis.call("DECRBY", KEYS[2], tonumber(ARGV[2]))
            
            -- 4. Registrar en stream de pendientes
            redis.call("XADD", KEYS[3], "*", "bet_id", ARGV[3], "user_id", ARGV[4], "match_id", ARGV[5], "selection", ARGV[6], "amount", ARGV[2], "odds", ARGV[1])
            
            return 1 -- OK
            "#,
        );

        let result: i64 = script
            .key(match_odds_key)
            .key(user_balance_key)
            .key(pending_bets_key)
            .arg(expected_odds.value_thousandths)
            .arg(amount.amount_cents)
            .arg(bet_id.0.to_string())
            .arg(user_id.0.to_string())
            .arg(match_id.0.to_string())
            .arg(selection.as_str())
            .invoke_async(&mut *conn)
            .await
            .map_err(map_redis_error)?;

        // volvemos lo que retorna el lua a tipos para el dominio
        match result {
            1 => Ok(()), // apuesta lograda, balance debitado
            -1 => { 
                // reportamos como default requerido el saldo de redis
                Err(DomainError::InsufficientFunds {
                    available: Money::new(0), // aqui se podria hacer un GET previo o posterior 
                    // pero rompería la pureza y la latencia del error path
                    required: amount,
                })
            }
            -2 => {
                // las cuotas no coinciden o no hay
                Err(DomainError::OddsChanged {
                    requested: expected_odds,
                    current: Odds::new(0), // el placeholder se re-fletchearia para informar al usuario
                    // pero por ahora se hace el reject atomic, que significa que la apuesta no se realiza
                })
            }
            _ => Err(DomainError::InfrastructureError(format!(
                "Código de error desconocido ({}) del script lua", result
            ))),
        }
    }
}