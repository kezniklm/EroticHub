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

const DEFAULT_VIDEO_DIRECTORY: &str = "./resources/videos";
const DEFAULT_THUMBNAILS_PATH: &str = "./resources/thumbnails";
const VIDEOS_DIRECTORY_KEY: &str = "VIDEO_DIRECTORY_PATH";
const THUMBNAIL_DIRECTORY_KEY: &str = "THUMBNAIL_DIRECTORY_PATH";

const DEFAULT_TEMP_DIRECTORY: &str = "temp";
const TEMP_DIRECTORY_KEY: &str = "TEMP_DIRECTORY_PATH";

pub fn init_configuration() -> anyhow::Result<Configuration> {
    let config_file = dotenvy::var(CONFIG_FILE_KEY).unwrap_or(String::from("./config.yaml"));

    let config = Config::builder()
        .add_source(config::File::with_name(config_file.as_str()))
        .build()?;
    let config = config.try_deserialize::<Configuration>()?;

    info!("Config {} was loaded!", config_file);
    Ok(config)
}

/// Function returns path to both video and thumbnail folder, where the files are stored.
///
/// # Returns
///
/// Tuple with:
/// - Path to video directory as String
/// - Path to thumbnails directory as String
pub fn get_video_thumbnail_dirs() -> (String, String) {
    let video = dotenvy::var(VIDEOS_DIRECTORY_KEY).unwrap_or(DEFAULT_VIDEO_DIRECTORY.to_string());
    let thumbnail =
        dotenvy::var(THUMBNAIL_DIRECTORY_KEY).unwrap_or(DEFAULT_THUMBNAILS_PATH.to_string());
    (video, thumbnail)
}

pub fn get_temp_directory_path() -> String {
    dotenvy::var(TEMP_DIRECTORY_KEY).unwrap_or(DEFAULT_TEMP_DIRECTORY.to_string())
}
