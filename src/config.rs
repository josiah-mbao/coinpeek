use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub symbols: Vec<String>,
    pub refresh_interval_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            symbols: vec![
                "BTCUSDT".to_string(),
                "ETHUSDT".to_string(),
                "BNBUSDT".to_string(),
                "ADAUSDT".to_string(),
                "SOLUSDT".to_string(),
                "DOTUSDT".to_string(),
                "DOGEUSDT".to_string(),
                "AVAXUSDT".to_string(),
                "LTCUSDT".to_string(),
                "LINKUSDT".to_string(),
            ],
            refresh_interval_seconds: 5,
        }
    }
}

impl Config {
    /// Load configuration from a JSON file, or create default if file doesn't exist
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = "coinpeek.json";

        if Path::new(config_path).exists() {
            let contents = fs::read_to_string(config_path)?;
            let config: Config = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            // Create default config file
            let default_config = Config::default();
            let json = serde_json::to_string_pretty(&default_config)?;
            fs::write(config_path, json)?;
            println!("Created default config file: coinpeek.json");
            println!("You can edit this file to customize which cryptocurrencies to track.");
            Ok(default_config)
        }
    }
}
