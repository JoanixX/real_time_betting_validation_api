use actix_governor::{GovernorConfig, GovernorConfigBuilder, KeyExtractor};
use actix_web::dev::ServiceRequest;
use actix_web::http::StatusCode;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct RealIpExtractor;

#[derive(Debug, Error)]
#[error("no se pudo extraer la ip del cliente")]
pub struct IpExtractionError;

impl actix_web::ResponseError for IpExtractionError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl KeyExtractor for RealIpExtractor {
    type Key = String;
    type KeyExtractionError = IpExtractionError;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        // obtenemos del connectinfo que resuelve el xforwarderfor
        // si un proxy confiable está frente a la aplicación
        if let Some(ip) = req.connection_info().realip_remote_addr() {
            // se limpia el string de cualquier puerto
            let ip_split = ip.split(':').next().unwrap_or(ip);
            return Ok(ip_split.to_string());
        }

        Err(IpExtractionError)
    }
}

// crea y configura el limitador de gobernadorcompartido
pub fn build_rate_limiter(
) -> GovernorConfig<RealIpExtractor, actix_governor::governor::middleware::StateInformationMiddleware>
{
    GovernorConfigBuilder::default()
        .requests_per_second(5000)
        .burst_size(5000)
        // en actix-governor 0.10, el keyextractor permite inyectar otra estructura
        .key_extractor(RealIpExtractor)
        .use_headers()
        .finish()
        .unwrap()
}
