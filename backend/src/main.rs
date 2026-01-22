use high_concurrency_api::config::get_configuration;
use high_concurrency_api::telemetry::{get_subscriber, init_subscriber};
use high_concurrency_api::Application;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Inicializar Telemetría (Logging)
    // Esto nos permite ver qué pasa en la app con logs estructurados en JSON
    let subscriber = get_subscriber(
        "high_concurrency_api".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    // 2. Cargar Configuración
    // Leemos archivos yaml y variables de entorno
    let configuration = get_configuration().expect("Falló la lectura de la configuración.");

    // 3. Construir la Aplicación
    // Aquí se inicializan pools de DB y listeners
    let application = Application::build(configuration).await?;

    // 4. Correr la Aplicación
    tracing::info!("Iniciando aplicación en el puerto {}", application.port());
    application.run_until_stopped().await?;

    Ok(())
}
