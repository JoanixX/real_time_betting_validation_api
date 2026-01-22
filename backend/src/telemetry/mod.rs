use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_subscriber::fmt::MakeWriter;

/// Compone múltiples capas en un `subscriber` de tracing.
///
/// # Notas de Implementación
/// 
/// Usamos `impl Subscriber` como tipo de retorno para evitar escribir 
/// el tipo complejo completo del subscriber.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync 
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // Filtro basado en variables de entorno (RUST_LOG)
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));
        
    // Capa de formato compatible con Bunyan (JSON)
    let formatting_layer = BunyanFormattingLayer::new(
        name,
        sink
    );
    
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Registra el subscriber como global para procesar spans.
///
/// Solo debe llamarse una vez.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirige logs estándar (log crates) a tracing
    LogTracer::init().expect("Falló al setear el logger");
    
    // Setea el subscriber global
    set_global_default(subscriber).expect("Falló al setear el subscriber");
}
