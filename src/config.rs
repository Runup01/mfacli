use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_pet")]
    pub pet: String,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default = "default_true")]
    pub show_weather: bool,
    #[serde(default = "default_true")]
    pub show_bazi: bool,
    #[serde(default = "default_true")]
    pub show_pet: bool,
    #[serde(default)]
    pub keychain: bool,
    #[serde(default = "default_qr_style")]
    pub qr_style: String,
}

fn default_qr_style() -> String {
    "half".to_string()
}

fn default_pet() -> String {
    "robot".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pet: default_pet(),
            city: None,
            show_weather: true,
            show_bazi: true,
            show_pet: true,
            keychain: false,
            qr_style: default_qr_style(),
        }
    }
}

impl Config {
    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir().ok_or("Cannot determine config directory")?;
        let dir = config_dir.join("mfa-cli");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("config.json"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .ok()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path()?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn set_pet(&mut self, pet: &str) -> Result<(), Box<dyn std::error::Error>> {
        let valid = ["robot", "dino", "cat", "ghost", "dragon"];
        if !valid.contains(&pet) {
            return Err(format!("Unknown pet '{}'. Choose from: {}", pet, valid.join(", ")).into());
        }
        self.pet = pet.to_string();
        self.save()
    }
}
