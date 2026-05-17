use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ide_command: String,
    pub alternative_ide_command: String, // e.g., antigravity
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ide_command: "code".to_string(),
            alternative_ide_command: "antigravity".to_string(),
        }
    }
}

pub fn get_config_path() -> PathBuf {
    PathBuf::from(".twigdrop")
}

pub fn load_config() -> Config {
    let path = get_config_path();
    if path.exists()
        && let Ok(content) = fs::read_to_string(path)
        && let Ok(config) = toml::from_str(&content)
    {
        return config;
    }
    // If not found or error, create default
    let config = Config::default();
    save_config(&config);
    config
}

pub fn save_config(config: &Config) {
    if let Ok(content) = toml::to_string_pretty(config) {
        let _ = fs::write(get_config_path(), content);
    }
}
