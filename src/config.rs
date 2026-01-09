use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use regex::Regex;

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
                "XRPUSDT".to_string(),
                "MATICUSDT".to_string(),
                "UNIUSDT".to_string(),
                "ALGOUSDT".to_string(),
                "VETUSDT".to_string(),
            ],
            refresh_interval_seconds: 3,
        }
    }
}

impl Config {
    /// Validate that a symbol follows proper cryptocurrency trading pair format
    pub fn is_valid_symbol(symbol: &str) -> bool {
        // Binance symbols are typically 3-10 uppercase letters, followed by 3-4 uppercase letters
        // Examples: BTCUSDT, ETHBTC, ADAUSDT, etc.
        let symbol_regex = Regex::new(r"^[A-Z]{3,10}[A-Z]{3,4}$").unwrap();
        symbol_regex.is_match(symbol)
    }

    /// Validate refresh interval is reasonable (not too fast to avoid rate limits)
    pub fn is_valid_refresh_interval(interval: u64) -> bool {
        // Allow 1-300 seconds (5 minutes max)
        interval >= 1 && interval <= 300
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate symbols
        if self.symbols.is_empty() {
            return Err("At least one symbol must be configured".to_string());
        }

        if self.symbols.len() > 50 {
            return Err("Too many symbols configured (max 50)".to_string());
        }

        // Check for duplicates
        let mut seen = std::collections::HashSet::new();
        for symbol in &self.symbols {
            if !seen.insert(symbol.clone()) {
                return Err(format!("Duplicate symbol found: {}", symbol));
            }

            if !Self::is_valid_symbol(symbol) {
                return Err(format!("Invalid symbol format: {}. Must be uppercase letters only, like 'BTCUSDT'", symbol));
            }
        }

        // Validate refresh interval
        if !Self::is_valid_refresh_interval(self.refresh_interval_seconds) {
            return Err(format!("Invalid refresh interval: {}. Must be between 1-300 seconds", self.refresh_interval_seconds));
        }

        Ok(())
    }

    /// Load configuration from a JSON file, or create default if file doesn't exist
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = "coinpeek.json";

        let config = if Path::new(config_path).exists() {
            let contents = fs::read_to_string(config_path)?;
            let config: Config = serde_json::from_str(&contents)?;
            config
        } else {
            // Create default config file
            let default_config = Config::default();
            let json = serde_json::to_string_pretty(&default_config)?;
            fs::write(config_path, json)?;
            println!("Created default config file: coinpeek.json");
            println!("You can edit this file to customize which cryptocurrencies to track.");
            default_config
        };

        // Validate the loaded configuration
        config.validate().map_err(|e| {
            format!("Configuration validation failed: {}. Please fix coinpeek.json", e)
        })?;

        Ok(config)
    }
}
