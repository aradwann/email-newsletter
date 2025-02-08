use config::{Config, ConfigError, Environment, File};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use std::env;

pub enum Enviroment {
    Development,
    Production,
}
impl Enviroment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Enviroment::Development => "development",
            Enviroment::Production => "production",
        }
    }
}

impl TryFrom<&str> for Enviroment {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "development" => Ok(Enviroment::Development),
            "production" => Ok(Enviroment::Production),
            _ => Err("Unsupported environment".into()),
        }
    }
}
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .username(&self.username)
            .password(&self.password)
            .host(&self.host)
            .port(self.port)
            .ssl_mode(match self.require_ssl {
                true => PgSslMode::Require,
                false => PgSslMode::Prefer,
            })
    }
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let run_mode = env::var("APP_ENV").unwrap_or_else(|_| "development".into());

    let settings = Config::builder()
        .add_source(File::with_name("configuration/base"))
        .add_source(File::with_name(&format!("configuration/{run_mode}")).required(false))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(Environment::with_prefix("app"))
        .build()?;

    settings.try_deserialize()
}
