use std::env;

use config::{Config, ConfigError, Environment, File};

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
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
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

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}
