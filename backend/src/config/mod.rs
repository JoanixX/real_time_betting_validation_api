use config::{Config, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::convert::{TryFrom, TryInto};

#[derive(Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    // expresion cron para el job de reconciliacion de balances
    #[serde(default = "default_reconciliation_cron")]
    pub reconciliation_cron: String,
}

fn default_reconciliation_cron() -> String {
    "0 * * * * *".to_string()
}

#[derive(Deserialize)]
pub struct RedisSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub upstash_redis_rest_url: Option<String>,
    pub upstash_redis_rest_token: Option<Secret<String>>,
}

impl RedisSettings {
    pub fn connection_string(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }

    pub fn use_upstash(&self) -> bool {
        self.upstash_redis_rest_url.is_some() && self.upstash_redis_rest_token.is_some()
    }
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Falló al determinar el directorio actual");
    let configuration_directory = base_path.join("configuration");

    // Cargamos el .env apropiado según el entorno
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Falló al parsear APP_ENVIRONMENT.");

    let env_file = match environment {
        Environment::Local => ".env",
        Environment::Production => ".env.production",
    };
    let env_path = base_path.join(env_file);
    if env_path.exists() {
        dotenvy::from_path(&env_path).ok();
    }

    let environment_filename = format!("{}.yaml", environment.as_str());

    // cargamos: base.yaml -> entorno.yaml -> variables de entorno con prefijo APP__
    let mut settings: Settings = Config::builder()
        .add_source(File::from(configuration_directory.join("base.yaml")))
        .add_source(File::from(configuration_directory.join(environment_filename)).required(false))
        .add_source(config::Environment::with_prefix("APP").separator("__"))
        .build()?
        .try_deserialize()?;

    // upstash se inyecta directo desde env vars (no usan prefijo APP_)
    if let Ok(url) = std::env::var("UPSTASH_REDIS_REST_URL") {
        settings.redis.upstash_redis_rest_url = Some(url);
    }
    if let Ok(token) = std::env::var("UPSTASH_REDIS_REST_TOKEN") {
        settings.redis.upstash_redis_rest_token = Some(Secret::new(token));
    }

    // si existe DATABASE_URL, la parseamos y sobreescribimos los campos del struct
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        if let Some(parsed) = parse_database_url(&db_url) {
            settings.database.host = parsed.host;
            settings.database.port = parsed.port;
            settings.database.username = parsed.username;
            settings.database.password = Secret::new(parsed.password);
            settings.database.database_name = parsed.database_name;
            if db_url.contains("sslmode=require") {
                settings.database.require_ssl = true;
            }
        }
    }

    Ok(settings)
}

struct ParsedDbUrl {
    host: String,
    port: u16,
    username: String,
    password: String,
    database_name: String,
}

fn parse_database_url(url: &str) -> Option<ParsedDbUrl> {
    let url = url.split('?').next()?;
    let url = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))?;

    let (credentials, host_part) = url.split_once('@')?;
    let (username, password) = credentials.split_once(':')?;
    let (host_port, database_name) = host_part.split_once('/')?;

    let (host, port_str) = if host_port.contains(':') {
        let (h, p) = host_port.rsplit_once(':')?;
        (h, p)
    } else {
        (host_port, "5432")
    };

    Some(ParsedDbUrl {
        host: host.to_string(),
        port: port_str.parse().unwrap_or(5432),
        username: username.to_string(),
        password: password.to_string(),
        database_name: database_name.to_string(),
    })
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} no es un entorno soportado. Usa 'local' o 'production'.",
            )),
        }
    }
}
