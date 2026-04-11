use high_concurrency_api::config::get_configuration;
use high_concurrency_api::telemetry::{get_subscriber, init_subscriber};
use high_concurrency_api::Application;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::never(".", "test_logs.txt");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    use tracing_subscriber::fmt::writer::MakeWriterExt;
    let subscriber = get_subscriber(
        "high_concurrency_api".into(),
        "info".into(),
        std::io::stdout.and(non_blocking),
    );
    init_subscriber(subscriber);

    // 2. cargar configuración desde yaml y variables de entorno
    let configuration = get_configuration().expect("Falló la lectura de la configuración");

    // 3. construir la app (pools de db, listeners, cache)
    let application = Application::build(configuration).await?;

    // 4. Correr la Aplicación
    tracing::info!("Iniciando aplicación en el puerto {}", application.port());
    application.run_until_stopped().await?;

    Ok(())
}
