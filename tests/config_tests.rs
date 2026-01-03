use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

use coinpeek::config::Config;

#[test]
fn test_config_default_values() {
    let default_config = Config::default();

    assert_eq!(default_config.symbols.len(), 10);
    assert!(default_config.symbols.contains(&"BTCUSDT".to_string()));
    assert!(default_config.symbols.contains(&"ETHUSDT".to_string()));
    assert_eq!(default_config.refresh_interval_seconds, 5);
}

#[test]
fn test_config_load_creates_default_when_missing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Change to temp directory for this test
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    // Ensure no config file exists
    assert!(!Path::new("coinpeek.json").exists());

    // Load config (should create default)
    let config = Config::load().unwrap();

    // Should have created the file
    assert!(Path::new("coinpeek.json").exists());

    // Should have default values
    assert!(config.symbols.len() >= 2); // At least BTCUSDT and ETHUSDT
    assert!(config.symbols.contains(&"BTCUSDT".to_string()));
    assert!(config.symbols.contains(&"ETHUSDT".to_string()));
    assert_eq!(config.refresh_interval_seconds, 5);

    // Verify the JSON file content
    let content = fs::read_to_string("coinpeek.json").unwrap();
    let parsed_config: Config = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed_config.symbols, config.symbols);
    assert_eq!(parsed_config.refresh_interval_seconds, config.refresh_interval_seconds);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_config_load_existing_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Change to temp directory for this test
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_path).unwrap();

    // Create a custom config file
    let custom_config = Config {
        symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        refresh_interval_seconds: 10,
    };

    let json = serde_json::to_string_pretty(&custom_config).unwrap();
    fs::write("coinpeek.json", json).unwrap();

    // Load config
    let loaded_config = Config::load().unwrap();

    // Should match the custom config
    assert_eq!(loaded_config.symbols, custom_config.symbols);
    assert_eq!(loaded_config.refresh_interval_seconds, custom_config.refresh_interval_seconds);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_config_json_serialization() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        refresh_interval_seconds: 15,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&config).unwrap();

    // Deserialize back
    let deserialized: Config = serde_json::from_str(&json).unwrap();

    // Should be identical
    assert_eq!(config.symbols, deserialized.symbols);
    assert_eq!(config.refresh_interval_seconds, deserialized.refresh_interval_seconds);
}

#[test]
fn test_config_validation() {
    // Test with empty symbols (should still work)
    let config = Config {
        symbols: vec![],
        refresh_interval_seconds: 30,
    };

    // Should serialize/deserialize fine
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    assert!(deserialized.symbols.is_empty());

    // Test with very large refresh interval
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 3600, // 1 hour
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.refresh_interval_seconds, 3600);
}

#[test]
fn test_config_pretty_json_formatting() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 5,
    };

    let json = serde_json::to_string_pretty(&config).unwrap();

    // Should contain newlines and proper indentation
    assert!(json.contains('\n'));
    assert!(json.contains("  ")); // indentation

    // Should be valid JSON
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.symbols, config.symbols);
}
