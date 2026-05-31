use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderConfig {
    pub model: String,
    pub api_key: String, // Obfuscated
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ide_command: String,
    pub alternative_ide_command: String, // e.g., antigravity
    pub last_primary_mode: usize,        // 0: Branches, 1: Files
    pub ai_provider: String,
    pub providers: std::collections::HashMap<String, ProviderConfig>,
    pub enable_animations: bool,
    pub default_sidebar_width: usize,
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = std::collections::HashMap::new();

        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                model: "llama3".to_string(),
                api_key: String::new(),
                url: "http://localhost:11434".to_string(),
            },
        );

        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                model: "gpt-4o".to_string(),
                api_key: String::new(),
                url: "https://api.openai.com/v1".to_string(),
            },
        );

        providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                model: "claude-3-5-sonnet-latest".to_string(),
                api_key: String::new(),
                url: "https://api.anthropic.com".to_string(),
            },
        );

        providers.insert(
            "google".to_string(),
            ProviderConfig {
                model: "gemini-1.5-pro".to_string(),
                api_key: String::new(),
                url: "https://generativelanguage.googleapis.com".to_string(),
            },
        );

        Self {
            ide_command: "code".to_string(),
            alternative_ide_command: "antigravity".to_string(),
            last_primary_mode: 0,
            ai_provider: "ollama".to_string(),
            providers,
            enable_animations: false,
            default_sidebar_width: 30,
        }
    }
}

impl Config {
    pub fn current_provider(&self) -> &ProviderConfig {
        self.providers
            .get(&self.ai_provider)
            .expect("Provider map must contain the current ai_provider")
    }

    pub fn current_provider_mut(&mut self) -> &mut ProviderConfig {
        if !self.providers.contains_key(&self.ai_provider) {
            self.providers.insert(
                self.ai_provider.clone(),
                ProviderConfig {
                    model: String::new(),
                    api_key: String::new(),
                    url: String::new(),
                },
            );
        }
        self.providers.get_mut(&self.ai_provider).unwrap()
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

pub const PROVIDER_ARCHETYPES: &[&str] = &["ollama", "openai", "anthropic", "google", "cohere"];

pub const OPENAI_MODELS: &[&str] = &["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-3.5-turbo"];
pub const ANTHROPIC_MODELS: &[&str] = &[
    "claude-3-5-sonnet-latest",
    "claude-3-opus-latest",
    "claude-3-haiku-20240307",
];
pub const GOOGLE_MODELS: &[&str] = &["gemini-1.5-pro", "gemini-1.5-flash", "gemini-1.0-pro"];

pub async fn fetch_ollama_models(url: &str) -> Vec<String> {
    let client = reqwest::Client::new();
    let res = client.get(format!("{}/api/tags", url)).send().await;

    match res {
        Ok(response) => {
            if let Ok(json) = response.json::<serde_json::Value>().await
                && let Some(models) = json["models"].as_array()
            {
                return models
                    .iter()
                    .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                    .collect();
            }
            Vec::new()
        }
        Err(_) => Vec::new(),
    }
}

pub fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "odiador", "twigdrop").map(|dirs| {
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
