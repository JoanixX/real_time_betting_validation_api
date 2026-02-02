use redis::{AsyncCommands, Client};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use crate::config::RedisSettings;

#[derive(Clone)]
pub enum CacheStore {
    Tcp(Client),
    Rest {
        url: String,
        token: String,
        client: reqwest::Client,
    },
}

impl CacheStore {
    pub fn build(settings: &RedisSettings) -> Self {
        if settings.use_upstash() {
            let url = settings.upstash_redis_rest_url.as_ref().unwrap().clone();
            let token = settings.upstash_redis_rest_token.as_ref().unwrap().expose_secret().clone();
            let client = reqwest::Client::new();
            tracing::info!("Se usa el cliente de Upstash Redis REST");
            CacheStore::Rest { url, token, client }
        } else {
            let client = Client::open(settings.connection_string())
                .expect("Falló al crearse el Redis TCP client");
            tracing::info!("Se usa el cliente de Redis TCP local");
            CacheStore::Tcp(client)
        }
    }

    pub async fn set(&self, key: &str, value: &str, expire_secs: usize) -> anyhow::Result<()> {
        match self {
            CacheStore::Tcp(client) => {
                let mut conn = client.get_async_connection().await?;
                conn.set_ex(key, value, expire_secs).await?;
                Ok(())
            }
            CacheStore::Rest { url, token, client } => {
                let endpoint = format!("{}/SET/{}/{}/EX/{}", url, key, value, expire_secs);
                let mut headers = HeaderMap::new();
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", token))?,
                );

                let response = client
                    .post(&endpoint)
                    .headers(headers)
                    .send()
                    .await?;

                if response.status().is_success() {
                    Ok(())
                } else {
                    let err_text = response.text().await?;
                    anyhow::bail!("Upstash error: {}", err_text)
                }
            }
        }
    }

    pub async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        match self {
            CacheStore::Tcp(client) => {
                let mut conn = client.get_async_connection().await?;
                let val: Option<String> = conn.get(key).await?;
                Ok(val)
            }
            CacheStore::Rest { url, token, client } => {
                let endpoint = format!("{}/GET/{}", url, key);
                let mut headers = HeaderMap::new();
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", token))?,
                );

                let response = client
                    .get(&endpoint)
                    .headers(headers)
                    .send()
                    .await?;

                #[derive(Deserialize)]
                struct UpstashResponse {
                    result: Option<String>,
                }

                if response.status().is_success() {
                    let result: UpstashResponse = response.json().await?;
                    Ok(result.result)
                } else {
                    let err_text = response.text().await?;
                    anyhow::bail!("Upstash error: {}", err_text)
                }
            }
        }
    }
}