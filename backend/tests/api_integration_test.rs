use high_concurrency_api::config::get_configuration;
use high_concurrency_api::telemetry::{get_subscriber, init_subscriber};
use high_concurrency_api::Application;
use once_cell::sync::Lazy;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::{postgres::Postgres, redis::Redis};
use sqlx::PgPool;
use std::time::Duration;
use secrecy::ExposeSecret;

// aseguramos que el log se inicializa solo una vez para todos los tests
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

#[tokio::test]
async fn place_bet_persists_to_postgres_via_redis_streams() {
    // 1. inicializamos el setup global logger
    Lazy::force(&TRACING);

    // 2. levantamos los contenedores de prueba E2E
    let pg_node = Postgres::default().with_tag("15-alpine").start().await.unwrap();
    let redis_node = Redis::default().with_tag("7-alpine").start().await.unwrap();

    let db_host = pg_node.get_host().await.unwrap();
    let db_port = pg_node.get_host_port_ipv4(5432).await.unwrap();

    let redis_host = redis_node.get_host().await.unwrap();
    let redis_port = redis_node.get_host_port_ipv4(6379).await.unwrap();

    // 3. preparamos los settings reales
    let mut config = get_configuration().expect("Falló al leer la configuración base.");

    // sobrescribimos con los puertos desde docker
    config.database.host = db_host.to_string();
    config.database.port = db_port;
    config.database.username = "postgres".to_string();
    config.database.password = secrecy::Secret::new("postgres".to_string());
    config.database.database_name = "postgres".to_string();

    config.redis.host = redis_host.to_string();
    config.redis.port = redis_port;
    config.redis.upstash_redis_rest_url = None;
    config.redis.upstash_redis_rest_token = None;

    // levantamos el webserver en un bind port aleatorio libre en el OS
    config.application.port = 0;

    // 4. corremos migraciones de sqlx en el contenedor efímero de postgres
    let db_pool = PgPool::connect_with(
        sqlx::postgres::PgConnectOptions::new()
            .host(&config.database.host)
            .port(config.database.port)
            .username(&config.database.username)
            .password(config.database.password.expose_secret())
            .database(&config.database.database_name)
    ).await.expect("Falló la conexión al Postgres testcontainer.");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Falló al ejecutar las migraciones en el testcontainer.");

    // inyectamos un usuario valido con saldo (schema real: tabla users con balance integrado)
    let user_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, name, balance)
        VALUES ($1, 'test@test.com', 'hash', 'Test User', 100000)
        "#)
        .bind(user_id)
        .execute(&db_pool).await.unwrap();

    // 5. levantamos la app real en plano asincrono
    let application = Application::build(config).await.expect("Falló build app.");
    let app_port = application.port();
    let server_task = tokio::spawn(application.run_until_stopped());

    // 6. cliente HTTP para el test
    let client = reqwest::Client::new();

    let match_id = uuid::Uuid::new_v4();

    // 7. POST al endpoint de bets
    // selection es requerido por el DTO desde que se agrego el campo al backend
    // odds en milésimas (1500 = 1.50), amount en centavos
    let response = client.post(&format!("http://127.0.0.1:{}/bets", app_port))
        .json(&serde_json::json!({
            "user_id": user_id,
            "match_id": match_id,
            "selection": "HomeWin",
            "amount": 500,
            "odds": 1500,
        }))
        .send()
        .await
        .expect("Error al lanzar petición HTTP.");

    assert_eq!(response.status().as_u16(), 201, "El server no retornó 201 Created");

    let json_resp: serde_json::Value = response.json().await.unwrap();
    let bet_id_str = json_resp["bet_id"].as_str().unwrap();
    let bet_uuid = uuid::Uuid::parse_str(bet_id_str).unwrap();

    // 8. polling a postgres — 4 segundos total para runners lentos de CI
    let max_retries = 400;
    let mut current_retry = 0;
    let mut bet_persisted = false;

    while current_retry < max_retries {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT count(*) FROM bets WHERE id = $1"
        )
        .bind(bet_uuid)
        .fetch_one(&db_pool)
        .await
        .expect("Error haciendo polling a la DB Postgres.");

        if count.unwrap_or(0) > 0 {
            bet_persisted = true;
            break;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
        current_retry += 1;
    }

    assert!(
        bet_persisted,
        "Timeout: El worker de Redis Streams no persistió la apuesta en Postgres después de 4 segundos."
    );

    // 9. graceful shutdown del test
    server_task.abort();
}