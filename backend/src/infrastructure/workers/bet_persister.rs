use chrono::Utc;
use deadpool_redis::redis::streams::{StreamReadOptions, StreamReadReply};
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::redis::ErrorKind;
use deadpool_redis::Pool;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;

const STREAM_KEY: &str = "bets_stream";
const GROUP_NAME: &str = "bets_cg";
const CONSUMER_NAME: &str = "persister_1";

// se levanta el consumer asincrono para asegurar la persistencia de las apuestas
pub fn spawn_bet_persister_worker(redis_pool: Pool, db_pool: PgPool) {
    tokio::spawn(async move {
        info!("Iniciando bet_persister...");

        let mut redis_conn = match redis_pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Worker falló al obtener conexión de Redis: {}", e);
                return;
            }
        };

        // crear consumer group si no existe (mkstream crea el stream si no existe)
        let group_created: deadpool_redis::redis::RedisResult<()> =
            deadpool_redis::redis::cmd("XGROUP")
                .arg("CREATE")
                .arg(STREAM_KEY)
                .arg(GROUP_NAME)
                .arg("$")
                .arg("MKSTREAM")
                .query_async(&mut *redis_conn)
                .await;

        match group_created {
            Ok(_) => info!(
                "Consumer Group '{}' creado en el stream '{}'",
                GROUP_NAME, STREAM_KEY
            ),
            Err(e) => {
                // error normal cuando el grupo ya existe (BUSYGROUP)
                if e.kind() == ErrorKind::ExtensionError && e.to_string().contains("BUSYGROUP") {
                    debug!("Consumer Group ya existe, procediendo.");
                } else {
                    error!("Fallo al crear el Consumer Group: {:?}", e);
                }
            }
        }

        // fase 1: leer el pel (pending entries list) para procesar mensajes solos
        info!("Leyendo PEL para procesar mensajes pendientes no confirmados...");
        let opts = StreamReadOptions::default()
            .group(GROUP_NAME, CONSUMER_NAME)
            .count(100);

        loop {
            // se lee el historial de pendientes del consumidor
            let pel_reply: deadpool_redis::redis::RedisResult<StreamReadReply> = redis_conn
                .xread_options(&[STREAM_KEY], &["0-0"], &opts)
                .await;

            match pel_reply {
                Ok(reply) => {
                    let mut has_pel_messages = false;
                    for stream_key in reply.keys {
                        for stream_id in stream_key.ids {
                            has_pel_messages = true;
                            process_and_ack_bet(
                                &mut redis_conn,
                                &db_pool,
                                stream_id.id,
                                stream_id.map,
                            )
                            .await;
                        }
                    }
                    if !has_pel_messages {
                        break;
                    }
                }
                Err(e) => {
                    error!("Error leyendo stream (PEL): {}", e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    if e.is_io_error() || e.is_connection_dropped() || e.to_string().contains("10054") {
                        if let Ok(new_conn) = redis_pool.get().await {
                            info!("Reconectado a Redis tras error en PEL.");
                            redis_conn = new_conn;
                            continue;
                        }
                    }
                    break;
                }
            }
        }

        // fase 2: loop infinito bloqueante leyendo nuevos mensajes
        info!("Escuchando nuevos mensajes del stream '{}'...", STREAM_KEY);
        let block_opts = StreamReadOptions::default()
            .group(GROUP_NAME, CONSUMER_NAME)
            .block(5000)
            .count(10); // batch de 10 max

        loop {
            let stream_reply: deadpool_redis::redis::RedisResult<StreamReadReply> = redis_conn
                .xread_options(&[STREAM_KEY], &[">"], &block_opts)
                .await;

            match stream_reply {
                Ok(reply) => {
                    for stream_key in reply.keys {
                        for stream_id in stream_key.ids {
                            process_and_ack_bet(
                                &mut redis_conn,
                                &db_pool,
                                stream_id.id,
                                stream_id.map,
                            )
                            .await;
                        }
                    }
                }
                Err(e) => {
                    // los timeouts de block en streams no devuelven un error
                    // tradicional, solo vacio
                    error!("Error leyendo stream (Nuevos mensajes): {}", e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    if e.is_io_error() || e.is_connection_dropped() || e.to_string().contains("10054") {
                        if let Ok(new_conn) = redis_pool.get().await {
                            info!("Reconectado a Redis tras error al leer nuevos mensajes.");
                            redis_conn = new_conn;
                        }
                    }
                }
            }
        }
    });
}

// procesa una apuesta de un mapa y persiste en postgres
async fn process_and_ack_bet(
    redis_conn: &mut deadpool_redis::Connection,
    db_pool: &PgPool,
    msg_id: String,
    map: HashMap<String, deadpool_redis::redis::Value>,
) {
    debug!("Procesando mensaje del stream con ID: {}", msg_id);

    // se mapea los valores de redis
    let parse_str = |key: &str| -> Option<String> {
        if let Some(deadpool_redis::redis::Value::Data(bytes)) = map.get(key) {
            String::from_utf8(bytes.clone()).ok()
        } else if let Some(deadpool_redis::redis::Value::Int(val)) = map.get(key) {
            Some(val.to_string())
        } else {
            None
        }
    };

    let bet_id_str = parse_str("bet_id").unwrap_or_default();
    let user_id_str = parse_str("user_id").unwrap_or_default();
    let match_id_str = parse_str("match_id").unwrap_or_default();
    let selection_str = parse_str("selection").unwrap_or_default();
    let amount_str = parse_str("amount").unwrap_or_default();
    let odds_str = parse_str("odds").unwrap_or_default();

    let bet_id = Uuid::parse_str(&bet_id_str).unwrap_or_default();
    let user_id = Uuid::parse_str(&user_id_str).unwrap_or_default();
    let match_id = Uuid::parse_str(&match_id_str).unwrap_or_default();

    // no usamos decimales para la base de datos, se hacen los montos son
    // en centavos y los odds en bigint o numeric sin escala

    // la bd fue migrada y el worker usa los string parseados a i64, asi que
    // se pasan directo. En la db debe manejarse como un numero entero
    // o un numeric escalado
    let amount_cents: i64 = amount_str.parse().unwrap_or(0);
    let odds_thousandths: i64 = odds_str.parse().unwrap_or(0);

    // validación basica para descartar mensajes erroneos
    if bet_id.is_nil() || user_id.is_nil() || selection_str.is_empty() {
        error!(
            "Mensaje {} tiene valores erróneos o nulos. Ignorando malformación. {:?}",
            msg_id, map
        );
        let _: deadpool_redis::redis::RedisResult<()> =
            redis_conn.xack(STREAM_KEY, GROUP_NAME, &[&msg_id]).await;
        return;
    }

    let res = sqlx::query(
        r#"
        INSERT INTO bets (id, user_id, match_id, selection, amount, odds, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(bet_id)
    .bind(user_id)
    .bind(match_id)
    .bind(selection_str)
    // guardamos los centavos y odds o cuotas en milesimas
    .bind(amount_cents)
    .bind(odds_thousandths)
    .bind("ACCEPTED")
    .bind(Utc::now())
    .execute(db_pool)
    .await;

    match res {
        Ok(_) => {
            // acknowledgment al stream solo si postgres hace el insert
            // o entra por el on conflict clause
            let ack_res: deadpool_redis::redis::RedisResult<()> =
                redis_conn.xack(STREAM_KEY, GROUP_NAME, &[&msg_id]).await;
            if let Err(e) = ack_res {
                error!(
                    "Insertada bet {} en Postgres pero falló al hacer XACK del mensaje {} ({})",
                    bet_id, msg_id, e
                );
            } else {
                info!("Apuesta {} persistida exitosamente. XACK enviado.", bet_id);
            }
        }
        Err(e) => {
            error!(
                "Error persistiendo apuesta {} en la base de datos: {:?}",
                bet_id, e
            );
            // si el query a db falla, deliberadamente no mandamos el xack
            // para que en un rescate se retenga la insercion
            // y asi se evita pérdida de eventos criticos
        }
    }
}
