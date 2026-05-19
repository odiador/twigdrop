use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ide_command: String,
    pub alternative_ide_command: String, // e.g., antigravity
    pub last_primary_mode: usize,        // 0: Branches, 1: Files
    pub ai_provider: String,
    pub ai_model: String,
    pub openai_api_key: String, // Obfuscated
    pub ollama_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ide_command: "code".to_string(),
            alternative_ide_command: "antigravity".to_string(),
            last_primary_mode: 0,
            ai_provider: "ollama".to_string(),
            ai_model: "llama3".to_string(),
            openai_api_key: String::new(),
            ollama_url: "http://localhost:11434".to_string(),
        }
    }
}

pub fn obfuscate(data: &str) -> String {
    let key = b"twigdrop_secret_key";
    let bytes = data.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    for (i, &byte) in bytes.iter().enumerate() {
        result.push(byte ^ key[i % key.len()]);
    }
    // Encode as hex for readability in toml
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn deobfuscate(data: &str) -> String {
    let key = b"twigdrop_secret_key";
    let mut bytes = Vec::new();
    for i in (0..data.len()).step_by(2) {
        if let Ok(byte) = u8::from_str_radix(&data[i..i + 2], 16) {
            bytes.push(byte);
        }
    }
    let mut result = Vec::with_capacity(bytes.len());
    for (i, &byte) in bytes.iter().enumerate() {
        result.push(byte ^ key[i % key.len()]);
    }
    String::from_utf8(result).unwrap_or_default()
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
