// archivo para las metricas personalizadas

use once_cell::sync::Lazy;
use prometheus::{IntCounter, IntGauge, Registry};

// Contador global de conexiones websocket activas
pub static BETTING_API_ACTIVE_WS_CONNECTIONS: Lazy<IntGauge> = Lazy::new(|| {
    IntGauge::new("betting_api_active_ws_connections", "Número de conexiones WebSocket activas en tiempo real")
        .expect("Error creando la métrica betting_api_active_ws_connections")
});

// Contador de apuestas aceptadas
pub static BETTING_API_BETS_PLACED_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    IntCounter::new("betting_api_bets_placed_total", "Número total de apuestas validadas y aceptadas")
        .expect("Error creando la métrica betting_api_bets_placed_total")
});

// Contador de apuestas rechazadas
pub static BETTING_API_BETS_REJECTED_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    IntCounter::new("betting_api_bets_rejected_total", "Número total de apuestas rechazadas por reglas de negocio")
        .expect("Error creando la métrica betting_api_bets_rejected_total")
});

// Ahora registramos las metricas en el Prometheus
pub fn register_custom_metrics(registry: &Registry) {
    registry.register(Box::new(BETTING_API_ACTIVE_WS_CONNECTIONS.clone()))
        .expect("Error registrando ws connections gauge");
    registry.register(Box::new(BETTING_API_BETS_PLACED_TOTAL.clone()))
        .expect("Error registrando bets placed counter");
    registry.register(Box::new(BETTING_API_BETS_REJECTED_TOTAL.clone()))
        .expect("Error registrando bets rejected counter");
}