pub mod config;
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod routes;
pub mod telemetry;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
use actix_cors::Cors;

// infraestructura
use crate::infrastructure::database;
use crate::infrastructure::cache::RedisCacheAdapter;
use crate::infrastructure::persistence::bet_repository::PostgresBetRepository;
use crate::infrastructure::persistence::user_repository::PostgresUserRepository;
use crate::infrastructure::redis_repo::RedisBettingStateRepository;
// es temporal para hacer compilar la inyección, esto idealmente 
// viviría en un modulo propio
struct PostgresMatchRepository;
#[async_trait::async_trait]
impl crate::domain::ports::MatchRepository for PostgresMatchRepository {
    async fn find_by_id(&self, _id: crate::domain::MatchId) -> Result<Option<crate::domain::SportMatch>, crate::domain::DomainError> {
        Ok(None) // mock por ahora
    }
}
use crate::infrastructure::security::Argon2Hasher;

// casos de uso
use crate::application::{PlaceBetUseCase, RegisterUserUseCase, LoginUserUseCase};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: config::Settings) -> Result<Self, anyhow::Error> {
        // adaptadores secundarios (infraestructura)
        let connection_pool = database::build_connection_pool(&configuration.database)
            .await
            .expect("Falló la conexión a postgres");

        let cache = RedisCacheAdapter::build(&configuration.redis);

        // pool de redis dedicado para alta concurrencia
        let redis_pool_config = deadpool_redis::Config::from_url(configuration.redis.connection_string());
        let redis_pool = redis_pool_config
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("Falló la creación del pool de redis");

        // inyección de dependencias
        // Construimos los casos de uso con sus puertos
        let _bet_repo = Arc::new(PostgresBetRepository::new(connection_pool.clone())); // aun disponible si otro UC lo necesita
        let _match_repo = Arc::new(PostgresMatchRepository);
        let user_repo = Arc::new(PostgresUserRepository::new(connection_pool.clone()));
        let hasher = Arc::new(Argon2Hasher::new());
        let cache_port: Arc<dyn domain::ports::CachePort> = Arc::new(cache);
        let bet_state_repo = Arc::new(RedisBettingStateRepository::new(redis_pool));

        let place_bet_uc = PlaceBetUseCase::new(bet_state_repo, cache_port);
        let register_uc = RegisterUserUseCase::new(user_repo.clone(), hasher.clone());
        let login_uc = LoginUserUseCase::new(user_repo, hasher);

        // direccion y puerto donde escucha el servidor
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        let server = run(listener, place_bet_uc, register_uc, login_uc)?;

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
    place_bet_uc: PlaceBetUseCase,
    register_uc: RegisterUserUseCase,
    login_uc: LoginUserUseCase,
) -> Result<Server, std::io::Error> {
    // envolvemos los casos de uso en Data para compartir entre threads de actix
    let place_bet_uc = web::Data::new(place_bet_uc);
    let register_uc = web::Data::new(register_uc);
    let login_uc = web::Data::new(login_uc);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin() // en prod hay que restringir esto
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default())
            .configure(routes::configure_routes)
            .app_data(place_bet_uc.clone())
            .app_data(register_uc.clone())
            .app_data(login_uc.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}