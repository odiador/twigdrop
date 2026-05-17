use serde::{Deserialize, Serialize};

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

pub fn load_config() -> Config {
    confy::load("twigdrop", None).unwrap_or_default()
}
