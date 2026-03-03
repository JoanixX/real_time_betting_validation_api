use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::domain::{UserId, MatchId};

// mensajes internos hacia la sesión ws
#[derive(Debug, Clone)]
pub enum WsMessage {
    // publicación de nuevas cuotas
    OddsUpdate { match_id: MatchId, odds: String },
    // cierre forzado del servidor
    Disconnect,
}

// estado en memoria por cliente ws
pub struct SessionState {
    // canal hacia la tarea del cliente
    pub sender: mpsc::UnboundedSender<WsMessage>,
    // partidos con suscripción activa
    pub subscriptions: dashmap::DashSet<MatchId>,
}

// manager concurrente con bajo overhead
#[derive(Clone)]
pub struct ConnectionManager {
    sessions: Arc<DashMap<UserId, SessionState>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }

    // registra nuevo cliente post-upgrade
    pub fn add_client(&self, user_id: UserId, sender: mpsc::UnboundedSender<WsMessage>) {
        self.sessions.insert(user_id, SessionState {
            sender,
            subscriptions: dashmap::DashSet::new(),
        });
        tracing::debug!("Usuario {} conectado al websocket", user_id);
    }

    // limpia sesión terminada de ram
    pub fn remove_client(&self, user_id: &UserId) {
        if self.sessions.remove(user_id).is_some() {
            tracing::debug!("Usuario {} desconectado y limpiado de RAM", user_id);
        }
    }

    // suscribe usuario a un partido
    pub fn subscribe_to_match(&self, user_id: &UserId, match_id: MatchId) {
        if let Some(session) = self.sessions.get(user_id) {
            session.subscriptions.insert(match_id);
            tracing::debug!("Usuario {} suscrito a cuotas del match {}", user_id, match_id);
        }
    }
    
    // remueve suscripción a partido
    pub fn unsubscribe_from_match(&self, user_id: &UserId, match_id: &MatchId) {
         if let Some(session) = self.sessions.get(user_id) {
            session.subscriptions.remove(match_id);
        }
    }

    // emite cuotas a los suscritos
    pub fn broadcast_odds_update(&self, match_id: MatchId, new_odds: &str) {
        let msg = WsMessage::OddsUpdate { 
            match_id, 
            odds: new_odds.to_string() 
        };

        // dashmap permite iterar sin bloquear escrituras concurrentes
        let mut disconnected_users = Vec::new();

        for entry in self.sessions.iter() {
            let user_id = entry.key();
            let session = entry.value();

            if session.subscriptions.contains(&match_id) {
                // si falla el envío, marcamos el socket para limpieza
                if session.sender.send(msg.clone()).is_err() {
                    disconnected_users.push(*user_id);
                }
            }
        }

        // recolección de basura de sockets muertos
        for dead_user in disconnected_users {
            self.remove_client(&dead_user);
        }
    }
}