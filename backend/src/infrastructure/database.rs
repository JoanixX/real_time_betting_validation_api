// Se hizo un pool de conexiones a postgres
// es solo nfraestructura pura para el wiring de lib.rs

use crate::config::DatabaseSettings;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn build_connection_pool(
    configuration: &DatabaseSettings,
) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(30))
        .max_connections(20)
        .min_connections(0) // Permite inicializar en 0 conexiones (sin atascos al inicio)
        .idle_timeout(std::time::Duration::from_secs(30))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect_lazy_with(
            configuration
                .connection_string()
                .expose_secret()
                .parse()
                .unwrap(),
        );

    // migraciones automáticas silenciosas al iniciar
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
         tracing::error!("Aviso: No se pudieron correr migraciones al inicio (probablemente Neon esté congelado): {:?}", e);
    }

    Ok(pool)
}