use actix_ws::{Message, Session};
use futures_util::StreamExt as _;
use std::time::{Duration, Instant};
use tokio::{sync::mpsc, time};

use crate::domain::UserId;
use super::manager::{ConnectionManager, WsMessage};

// cadencia de ping
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
// límite de inactividad
const CLIENT_TIMEOUT: Duration = Duration::from_secs(20);

// bucle asíncrono por socket conectado
pub async fn ws_session_loop(
    user_id: UserId,
    mut session: Session,
    mut msg_stream: actix_ws::MessageStream,
    manager: ConnectionManager,
    mut rx: mpsc::UnboundedReceiver<WsMessage>,
) {
    let mut last_heartbeat = Instant::now();
    let mut interval = time::interval(HEARTBEAT_INTERVAL);

    // se hace el bucle de lectura select
    loop {
        tokio::select! {
            // lanza ping y purga zombies
            _ = interval.tick() => {
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    tracing::debug!("Cliente websocket {} ha exedido el timeout (zombi). Desconectando.", user_id);
                    break;
                }

                if session.ping(b"").await.is_err() {
                    break;
                }
            }

            // aca se procesa frame entrante ws
            Some(Ok(msg)) = msg_stream.next() => {
                match msg {
                    Message::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        let _ = session.pong(&bytes).await;
                    }
                    Message::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }
                    Message::Text(text) => {
                        // enrutamiento del payload
                        let text_str = text.trim();
                        if text_str.starts_with("SUB:") {
                            let match_id_str = &text_str[4..];
                            if let Ok(parsed_uuid) = uuid::Uuid::parse_str(match_id_str) {
                                manager.subscribe_to_match(&user_id, crate::domain::MatchId(parsed_uuid));
                                let _ = session.text(format!("SUSCRITO OKEY: {}", parsed_uuid)).await;
                            }
                        } else if text_str.starts_with("UNSUB:") {
                            let match_id_str = &text_str[6..];
                            if let Ok(parsed_uuid) = uuid::Uuid::parse_str(match_id_str) {
                                manager.unsubscribe_from_match(&user_id, &crate::domain::MatchId(parsed_uuid));
                                let _ = session.text(format!("DESUSCRITO OKEY: {}", parsed_uuid)).await;
                            }
                        }
                    }
                    Message::Binary(_) => {
                        tracing::debug!("Recibido binary no soportado de {}", user_id);
                    }
                    Message::Close(reason) => {
                        tracing::debug!("Conexión websocket cerrada por el cliente {}: {:?}", user_id, reason);
                        break;
                    }
                    Message::Continuation(_) => {
                        break;
                    }
                    Message::Nop => {}
                }
            }

            // Retransmite comandos del manager
            Some(internal_msg) = rx.recv() => {
                match internal_msg {
                    WsMessage::OddsUpdate { match_id, odds } => {
                        // Empaqueta evento de dominio
                        let payload = serde_json::json!({
                            "type": "ODDS_UPDATE",
                            "match_id": match_id.0.to_string(),
                            "odds": odds
                        });
                        if session.text(payload.to_string()).await.is_err() {
                            break; // aborta si se cerro tcp
                        }
                    }
                    WsMessage::Disconnect => {
                        break;
                    }
                }
            }
        }
    }

    manager.remove_client(&user_id);
    let _ = session.close(None).await;
}