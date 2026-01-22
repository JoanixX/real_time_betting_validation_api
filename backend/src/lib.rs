pub mod config;
pub mod db;
pub mod domain;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod routes;
pub mod telemetry;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use tracing_actix_web::TracingLogger;
use actix_cors::Cors;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: config::Settings) -> Result<Self, anyhow::Error> {
        // Inicializamos el pool de base de datos
        let connection_pool = db::build_connection_pool(&configuration.database).await
            .expect("Falló la conexión a Postgres.");

        // Definimos la dirección y puerto
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        
        // Arrancamos el servidor
        let server = run(listener, connection_pool)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
) -> Result<Server, std::io::Error> {
    // Envolvemos el pool en un Data arc para compartirlo entre threads de actix
    let db_pool = web::Data::new(db_pool);
    
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin() // En producción deberías restringir esto
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default()) // Middleware de Logging Estructurado
            .route("/health_check", web::get().to(handlers::health_check))
            .configure(routes::configure_routes)
            .configure(routes::configure_routes)
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
