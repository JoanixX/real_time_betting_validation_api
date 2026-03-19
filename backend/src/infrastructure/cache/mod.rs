// Se creó un adaptador secundario con implementación redis/upstash del puerto CachePort
// soporta redis tcp local y upstash rest (producción serverless)

use crate::config::RedisSettings;
use crate::domain::ports::CachePort;
use crate::domain::DomainError;
use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use secrecy::ExposeSecret;
use serde::Deserialize;

#[derive(Clone)]
pub enum RedisCacheAdapter {
    Tcp(Client),
    Rest {
        url: String,
        token: String,
        client: reqwest::Client,
    },
}

impl RedisCacheAdapter {
    pub fn build(settings: &RedisSettings) -> Self {
        if settings.use_upstash() {
            let url = settings.upstash_redis_rest_url.as_ref().unwrap().clone();
            let token = settings
                .upstash_redis_rest_token
                .as_ref()
                .unwrap()
                .expose_secret()
                .clone();
            let client = reqwest::Client::new();
            tracing::info!("Se usa el cliente de Upstash Redis rest");
            RedisCacheAdapter::Rest { url, token, client }
        } else {
            let client = Client::open(settings.connection_string())
                .expect("Fallo al crearse el Redis TCP client");
            tracing::info!("Se usa el cliente de Redis TCP local");
            RedisCacheAdapter::Tcp(client)
        }
    }
}

#[async_trait]
impl CachePort for RedisCacheAdapter {
    async fn set(&self, key: &str, value: &str, expire_secs: usize) -> Result<(), DomainError> {
        match self {
            RedisCacheAdapter::Tcp(client) => {
                let mut conn = client
                    .get_async_connection()
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
                conn.set_ex::<_, _, ()>(key, value, expire_secs as u64)
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
                Ok(())
            }
            RedisCacheAdapter::Rest { url, token, client } => {
                let endpoint = format!("{url}/SET/{key}/{value}/EX/{expire_secs}");
                let mut headers = HeaderMap::new();
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {token}"))
                        .map_err(|e| DomainError::Internal(e.to_string()))?,
                );

                let response = client
                    .post(&endpoint)
                    .headers(headers)
                    .send()
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;

                if response.status().is_success() {
                    Ok(())
                } else {
                    let err_text = response
                        .text()
                        .await
                        .map_err(|e| DomainError::Internal(e.to_string()))?;
                    Err(DomainError::Internal(format!(
                        "Error en Upstash: {err_text}"
                    )))
                }
            }
        }
    }

    async fn get(&self, key: &str) -> Result<Option<String>, DomainError> {
        match self {
            RedisCacheAdapter::Tcp(client) => {
                let mut conn = client
                    .get_async_connection()
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
                let val: Option<String> = conn
                    .get(key)
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
                Ok(val)
            }
            RedisCacheAdapter::Rest { url, token, client } => {
                let endpoint = format!("{url}/GET/{key}");
                let mut headers = HeaderMap::new();
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {token}"))
                        .map_err(|e| DomainError::Internal(e.to_string()))?,
                );

                let response = client
                    .get(&endpoint)
                    .headers(headers)
                    .send()
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;

                #[derive(Deserialize)]
                struct UpstashResponse {
                    result: Option<String>,
                }

                if response.status().is_success() {
                    let result: UpstashResponse = response
                        .json()
                        .await
                        .map_err(|e| DomainError::Internal(e.to_string()))?;
                    Ok(result.result)
                } else {
                    let err_text = response
                        .text()
                        .await
                        .map_err(|e| DomainError::Internal(e.to_string()))?;
                    Err(DomainError::Internal(format!(
                        "Error en Upstash: {err_text}"
                    )))
                }
            }
        }
    }
}
