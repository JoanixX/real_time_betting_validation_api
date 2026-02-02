use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use secrecy::ExposeSecret;

use crate::config::DatabaseSettings;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.connection_string_without_db().expose_secret().parse().unwrap()) 
}

pub async fn build_connection_pool(configuration: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        // Configuración para alta concurrencia
        .max_connections(100) // Límite duro de conexiones
        .min_connections(5)   // Mantenemos algunas calientes
        .idle_timeout(std::time::Duration::from_secs(30)) // Cerramos conexiones ociosas
        .max_lifetime(std::time::Duration::from_secs(1800)) // Rotamos conexiones cada 30 min
        .connect_with(configuration.connection_string().expose_secret().parse().unwrap())
        .await?;

    // Ejecutamos migraciones automáticamente al iniciar
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Falló la ejecución de las migraciones: {:?}", e);
            e
        })?;

    Ok(pool)
}

pub mod cache;
