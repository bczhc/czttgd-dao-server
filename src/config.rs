use std::path::Path;
use figment::Figment;
use figment::providers::{Format, Toml};
use serde::Deserialize;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub listen_port: u16,
    pub mysql: MySql,
    pub logging: Option<Logging>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct MySql {
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Logging {
    pub file: Option<String>,
}

pub fn get_config<P: AsRef<Path>>(toml: P) -> figment::Result<Config> {
    Figment::new()
        .merge(Toml::file(toml))
        .extract()
}
