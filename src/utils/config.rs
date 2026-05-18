use directories::ProjectDirs;
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

pub fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "twigdrop", "twigdrop").map(|dirs| {
        let config_dir = dirs.config_dir();
        if !config_dir.exists() {
            let _ = fs::create_dir_all(config_dir);
        }
        config_dir.join("config.toml")
    })
}

pub fn load_config() -> Config {
    if let Some(path) = get_config_path()
        && path.exists()
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
    if let Some(path) = get_config_path()
        && let Ok(content) = toml::to_string_pretty(config)
    {
        let _ = fs::write(path, content);
    }
}
