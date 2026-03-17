use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_subscriber::fmt::MakeWriter;

pub mod metrics;

/// compone múltiples capas en un subscriber de tracing.
/// usamos `impl Subscriber` como retorno para no escribir el tipo completo.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync 
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // filtro basado en la variable RUST_LOG
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

/// registra el subscriber como global. solo se llama una vez.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirige logs estándar (log crates) a tracing
    LogTracer::init().expect("Falló al setear el logger");
    
    // Setea el subscriber global
    set_global_default(subscriber).expect("Falló al setear el subscriber");
}