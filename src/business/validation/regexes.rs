use once_cell::sync::Lazy;
use regex::Regex;

pub static RE_PATH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(/[a-zA-Z0-9-]+)+$").expect("Failed to create regex"));
