use crate::configuration::models::Configuration;
use config::Config;
use log::info;

pub mod api;
pub mod business;
pub mod common;
pub mod configuration;
pub mod persistence;
pub mod streamer;

const CONFIG_FILE_KEY: &str = "CONFIG_FILE_PATH";

pub fn init_configuration() -> anyhow::Result<Configuration> {
    let config_file = dotenvy::var(CONFIG_FILE_KEY).unwrap_or(String::from("./config.yaml"));

    let config = Config::builder()
        .add_source(config::File::with_name(config_file.as_str()))
        .build()?;
    let config = config.try_deserialize::<Configuration>()?;

    info!("Config {} was loaded!", config_file);
    Ok(config)
}
