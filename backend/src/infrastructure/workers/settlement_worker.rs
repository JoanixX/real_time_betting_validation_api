use deadpool_redis::Pool;
use deadpool_redis::redis::{AsyncCommands, ErrorKind};
use deadpool_redis::redis::streams::{StreamReadOptions, StreamReadReply};
use deadpool_redis::redis::Pipeline;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;
use tracing::{info, error, debug};
use std::collections::HashMap;
use crate::domain::BetSelection;

const STREAM_KEY: &str = "match_results_stream";
const GROUP_NAME: &str = "settlement_cg";
const CONSUMER_NAME: &str = "settlement_worker_1";

// Tracker de apuestas ganadoras y perdedoras
struct BetResultRecord {
    bet_id: Uuid,
    user_id: Uuid,
    new_status: &'static str,
    gain_cents: i64,
}

pub fn spawn_settlement_worker(redis_pool: Pool, db_pool: PgPool) {
    tokio::spawn(async move {
        info!("Iniciando settlement_worker...");

        let mut redis_conn = match redis_pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Settlement Worker falló al obtener conexión de Redis: {}", e);
                return;
            }
        };

        // creamos el connsumer group
        let group_created: deadpool_redis::redis::RedisResult<()> = deadpool_redis::redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(STREAM_KEY)
            .arg(GROUP_NAME)
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut *redis_conn)
            .await;

        match group_created {
            Ok(_) => info!("Consumer Group '{}' creado en el stream '{}'", GROUP_NAME, STREAM_KEY),
            Err(e) => {
                if e.kind() == ErrorKind::ExtensionError && e.to_string().contains("BUSYGROUP") {
                    debug!("Consumer Group {} ya existe.", GROUP_NAME);
                } else {
                    error!("Fallo al crear el Consumer Group {}: {:?}", GROUP_NAME, e);
                }
            }
        }

        // listas pendientes de entradas, o PEL
        info!("Settlement worker leyendo PEL...");
        let opts = StreamReadOptions::default().group(GROUP_NAME, CONSUMER_NAME).count(100);

        loop {
            let pel_reply: deadpool_redis::redis::RedisResult<StreamReadReply> = redis_conn
                .xread_options(&[STREAM_KEY], &["0-0"], &opts)
                .await;

            match pel_reply {
                Ok(reply) => {
                    let mut has_pel_messages = false;
                    for stream_key in reply.keys {
                        for stream_id in stream_key.ids {
                            has_pel_messages = true;
                            process_and_ack_match_result(&mut redis_conn, &db_pool, stream_id.id, stream_id.map).await;
                        }
                    }
                    if !has_pel_messages { break; }
                }
                Err(e) => {
                    error!("Error leyendo stream (PEL) en settlement: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    break;
                }
            }
        }

        info!("Settlement worker escuchando nuevos mensajes...");
        let block_opts = StreamReadOptions::default().group(GROUP_NAME, CONSUMER_NAME).block(5000).count(10);

        loop {
            let stream_reply: deadpool_redis::redis::RedisResult<StreamReadReply> = redis_conn
                .xread_options(&[STREAM_KEY], &[">"], &block_opts)
                .await;

            match stream_reply {
                Ok(reply) => {
                    for stream_key in reply.keys {
                        for stream_id in stream_key.ids {
                            process_and_ack_match_result(&mut redis_conn, &db_pool, stream_id.id, stream_id.map).await;
                        }
                    }
                }
                Err(e) => {
                    error!("Settlement Error leyendo stream (Nuevos mensajes): {}", e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    });
}

async fn process_and_ack_match_result(
    redis_conn: &mut deadpool_redis::Connection,
    db_pool: &PgPool,
    msg_id: String,
    map: HashMap<String, deadpool_redis::redis::Value>,
) {
    debug!("Procesando resultado de partido del stream ID: {}", msg_id);

    let parse_str = |key: &str| -> Option<String> {
        if let Some(deadpool_redis::redis::Value::Data(bytes)) = map.get(key) {
            String::from_utf8(bytes.clone()).ok()
        } else if let Some(deadpool_redis::redis::Value::Int(val)) = map.get(key) {
            Some(val.to_string())
        } else {
            None
        }
    };

    let match_id_str = parse_str("match_id").unwrap_or_default();
    let result_outcome_str = parse_str("result_outcome").unwrap_or_default();

    let match_id = match Uuid::parse_str(&match_id_str) {
        Ok(id) => id,
        Err(_) => {
            error!("Mensaje {} tiene un match_id inválido. Ignorando.", msg_id);
            let _: deadpool_redis::redis::RedisResult<()> = redis_conn.xack(STREAM_KEY, GROUP_NAME, &[&msg_id]).await;
            return;
        }
    };

    let winning_selection = match result_outcome_str.as_str() {
        "HomeWin" => BetSelection::HomeWin,
        "AwayWin" => BetSelection::AwayWin,
        "Draw" => BetSelection::Draw,
        _ => {
            error!("Mensaje {} tiene un result_outcome inválido ({}). Ignorando.", msg_id, result_outcome_str);
            let _: deadpool_redis::redis::RedisResult<()> = redis_conn.xack(STREAM_KEY, GROUP_NAME, &[&msg_id]).await;
            return;
        }
    };

    // 1. SELECT de apuestas aceptadas para el match_id 
    // estas se hacen en bigint
    let rows = match sqlx::query(
        r#"
        SELECT id, user_id, selection, amount, odds
        FROM bets 
        WHERE match_id = $1 AND status = 'ACCEPTED'
        "#,
    )
    .bind(match_id)
    .fetch_all(db_pool)
    .await {
        Ok(r) => r,
        Err(e) => {
            error!("Error al obtener apuestas para el match {}: {:?}", match_id, e);
            return; // fallamos silenciosamente sin ack para ser reintentado por pel
        }
    };

    if rows.is_empty() {
        debug!("Match {} no tiene apuestas ACCEPTED. Ackeando.", match_id);
        let _: deadpool_redis::redis::RedisResult<()> = redis_conn.xack(STREAM_KEY, GROUP_NAME, &[&msg_id]).await;
        return;
    }

    let mut records_to_update = Vec::with_capacity(rows.len());

    use sqlx::Row;
    for row in rows {
        let bet_id: Uuid = row.try_get("id").unwrap_or_default();
        let u_id: Uuid = row.try_get("user_id").unwrap_or_default();
        let selection: String = row.try_get("selection").unwrap_or_default();
        let amount: i64 = row.try_get("amount").unwrap_or_default();
        let odds: i64 = row.try_get("odds").unwrap_or_default();

        let is_winner = selection == winning_selection.as_str();
        let new_status = if is_winner { "WON" } else { "LOST" };
        
        // amount está en cents y odds está en milesimas
        let gain_cents = if is_winner {
            (amount * odds) / 1000
        } else {
            0
        };

        records_to_update.push(BetResultRecord {
            bet_id,
            user_id: u_id,
            new_status,
            gain_cents,
        });
    }

    // preparamos los vectores para el unnest
    let mut bet_ids = Vec::with_capacity(records_to_update.len());
    let mut bet_statuses = Vec::with_capacity(records_to_update.len());
    
    let mut user_ids_gains = Vec::new();
    let mut actual_gains = Vec::new();

    for record in &records_to_update {
        bet_ids.push(record.bet_id);
        bet_statuses.push(record.new_status.to_string());

        if record.gain_cents > 0 {
            user_ids_gains.push(record.user_id);
            actual_gains.push(record.gain_cents);
        }
    }

    // 2. hacemos la trnsaccion a la bd con el settlement ACID
    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            error!("Fallo al iniciar transacción para Match {}: {:?}", match_id, e);
            return;
        }
    };

    // verificacion de idempotencia estricta
    let idempotency_res = sqlx::query(
        r#"
        INSERT INTO processed_match_results (match_id) 
        VALUES ($1) 
        ON CONFLICT (match_id) DO NOTHING
        "#
    )
    .bind(match_id)
    .execute(&mut *tx)
    .await;

    match idempotency_res {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let _ = tx.rollback().await;
                tracing::warn!("Idempotency trigger: Match {} already processed. Skipping settlement.", match_id);
                // saltamos directo al xack
                let _: deadpool_redis::redis::RedisResult<()> = redis_conn.xack(STREAM_KEY, GROUP_NAME, &[&msg_id]).await;
                return;
            }
        }
        Err(e) => {
            let _ = tx.rollback().await;
            error!("Fallo al insertar llave de idempotencia para Match {}: {:?}", match_id, e);
            return;
        }
    }

    // hacemos el bulk update para el bets
    if let Err(e) = sqlx::query(
        r#"
        UPDATE bets SET status = u.new_status
        FROM (SELECT unnest($1::uuid[]) as id, unnest($2::text[]) as new_status) as u
        WHERE bets.id = u.id
        "#
    )
    .bind(&bet_ids)
    .bind(&bet_statuses)
    .execute(&mut *tx)
    .await {
        error!("Fallo en Bulk Update de Bets para Match {}: {:?}", match_id, e);
        return;
    };

    // actualizamos el bulk solo a ganadores
    if !user_ids_gains.is_empty() {
        if let Err(e) = sqlx::query(
            r#"
            UPDATE users SET balance = balance + u.gain
            FROM (SELECT unnest($1::uuid[]) as id, unnest($2::bigint[]) as gain) as u
            WHERE users.id = u.id
            "#
        )
        .bind(&user_ids_gains)
        .bind(&actual_gains)
        .execute(&mut *tx)
        .await {
            error!("Fallo en Bulk Update de User Balances para Match {}: {:?}", match_id, e);
            return;
        };
    }

    if let Err(e) = tx.commit().await {
        error!("Fallo al comitear la transacción de Settlement para Match {}: {:?}", match_id, e);
        return;
    }

    // 3. redis pipeline para actualizacion del saldo en memoria
    // mantenemos la sincronización para la api rapida en lecturas
    if !user_ids_gains.is_empty() {
        let mut pipe = deadpool_redis::redis::pipe();
        pipe.atomic(); // con esto aseguramos que el batch de instrucciones 
        // vaya al servidor como un paquete atomico de multi o exec
        
        for record in records_to_update.iter().filter(|r| r.gain_cents > 0) {
            let user_balance_key = format!("user:{}:balance", record.user_id);
            pipe.incr(&user_balance_key, record.gain_cents).ignore();
        }

        let pipe_res: deadpool_redis::redis::RedisResult<()> = pipe.query_async(&mut *redis_conn).await;
        if let Err(e) = pipe_res {
            // un error aca es molesto, ya se comiteo a la bd asi que para aplicaciones
            // asi se podria usar un log de reconciliacion asincrona o compensacion.
            error!("CRITICO: DB comiteo la ganancia para match {} pero Redis Pipeline falló: {:?}", match_id, e);
        }
    }

    // 4. xack final
    let ack_res: deadpool_redis::redis::RedisResult<()> = redis_conn.xack(STREAM_KEY, GROUP_NAME, &[&msg_id]).await;
    if let Err(e) = ack_res {
        error!("Liquidado Match {} en SQL, pero falló al hacer XACK del mensaje {} ({})", match_id, msg_id, e);
    } else {
        info!("Match {} liquidado exitosamente ({} apuestas procesadas). XACK enviado.", match_id, bet_ids.len());
    }
}