use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio_test::block_on;

use coinpeek::database::{Database, DatabaseStats};
use coinpeek::binance::{PriceInfo, Candle};

#[test]
fn test_database_initialization() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    block_on(async {
        let db = Database::new(db_path).await;
        assert!(db.is_ok(), "Database should initialize successfully");
    });
}

#[test]
fn test_store_and_retrieve_price_info() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    block_on(async {
        let db = Database::new(db_path).await.unwrap();

        // Create test price info
        let price_info = PriceInfo {
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            price_change_percent: 2.5,
            volume: 1000.0,
            high_24h: 51000.0,
            low_24h: 49000.0,
            prev_close_price: 48750.0,
        };

        // Store price info
        let store_result = db.store_price_info(&price_info).await;
        assert!(store_result.is_ok(), "Should store price info successfully");

        // Retrieve latest price
        let retrieved = db.get_latest_price("BTCUSDT").await.unwrap();
        assert!(retrieved.is_some(), "Should retrieve stored price");

        let retrieved_price = retrieved.unwrap();
        assert_eq!(retrieved_price.symbol, "BTCUSDT");
        assert_eq!(retrieved_price.price, 50000.0);
        assert_eq!(retrieved_price.price_change_percent, 2.5);
    });
}

#[test]
fn test_bulk_price_storage() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    block_on(async {
        let db = Database::new(db_path).await.unwrap();

        let price_infos = vec![
            PriceInfo {
                symbol: "BTCUSDT".to_string(),
                price: 50000.0,
                price_change_percent: 2.5,
                volume: 1000.0,
                high_24h: 51000.0,
                low_24h: 49000.0,
                prev_close_price: 48750.0,
            },
            PriceInfo {
                symbol: "ETHUSDT".to_string(),
                price: 3000.0,
                price_change_percent: -1.2,
                volume: 500.0,
                high_24h: 3100.0,
                low_24h: 2900.0,
                prev_close_price: 3036.0,
            },
        ];

        // Store multiple price infos
        let store_result = db.store_price_infos(&price_infos).await;
        assert!(store_result.is_ok(), "Should store multiple price infos successfully");

        // Verify both prices were stored
        let btc_price = db.get_latest_price("BTCUSDT").await.unwrap().unwrap();
        let eth_price = db.get_latest_price("ETHUSDT").await.unwrap().unwrap();

        assert_eq!(btc_price.price, 50000.0);
        assert_eq!(eth_price.price, 3000.0);
    });
}

#[test]
fn test_candle_storage_and_retrieval() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    block_on(async {
        let db = Database::new(db_path).await.unwrap();

        let candles = vec![
            Candle {
                open: 50000.0,
                high: 51000.0,
                low: 49000.0,
                close: 50500.0,
                volume: 100.0,
                timestamp: 1640995200000, // 2022-01-01 00:00:00 UTC
            },
            Candle {
                open: 50500.0,
                high: 51500.0,
                low: 50000.0,
                close: 51000.0,
                volume: 120.0,
                timestamp: 1640995260000, // 2022-01-01 00:01:00 UTC
            },
        ];

        // Store candles
        let store_result = db.store_candles("BTCUSDT", "1m", &candles).await;
        assert!(store_result.is_ok(), "Should store candles successfully");

        // Retrieve candles
        let retrieved = db.get_candles("BTCUSDT", "1m", 10).await.unwrap();
        assert_eq!(retrieved.len(), 2, "Should retrieve both candles");

        // Verify data integrity
        assert_eq!(retrieved[0].open, 50000.0);
        assert_eq!(retrieved[0].close, 50500.0);
        assert_eq!(retrieved[1].high, 51500.0);
        assert_eq!(retrieved[1].volume, 120.0);
    });
}

#[test]
fn test_database_statistics() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    block_on(async {
        let db = Database::new(db_path).await.unwrap();

        // Initially empty
        let initial_stats = db.get_stats().await.unwrap();
        assert_eq!(initial_stats.price_records, 0);
        assert_eq!(initial_stats.candle_records, 0);

        // Add some data
        let price_info = PriceInfo {
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            price_change_percent: 2.5,
            volume: 1000.0,
            high_24h: 51000.0,
            low_24h: 49000.0,
            prev_close_price: 48750.0,
        };

        db.store_price_info(&price_info).await.unwrap();

        let candle = Candle {
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 100.0,
            timestamp: 1640995200000,
        };

        db.store_candles("BTCUSDT", "1m", &[candle]).await.unwrap();

        // Check updated stats
        let updated_stats = db.get_stats().await.unwrap();
        assert_eq!(updated_stats.price_records, 1);
        assert_eq!(updated_stats.candle_records, 1);
        assert!(updated_stats.database_size_bytes > 0, "Database should have non-zero size");
    });
}

#[test]
fn test_sync_metadata_operations() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    block_on(async {
        let db = Database::new(db_path).await.unwrap();

        // Initially no metadata
        let initial = db.get_sync_metadata("last_price_sync").await.unwrap();
        assert!(initial.is_none(), "Should have no initial metadata");

        // Set metadata
        db.update_sync_metadata("last_price_sync", "1640995200000").await.unwrap();

        // Retrieve metadata
        let retrieved = db.get_sync_metadata("last_price_sync").await.unwrap();
        assert_eq!(retrieved, Some("1640995200000".to_string()));

        // Update existing metadata
        db.update_sync_metadata("last_price_sync", "1640995260000").await.unwrap();
        let updated = db.get_sync_metadata("last_price_sync").await.unwrap();
        assert_eq!(updated, Some("1640995260000".to_string()));
    });
}

#[test]
fn test_get_active_symbols() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    block_on(async {
        let db = Database::new(db_path).await.unwrap();

        // Initially no active symbols
        let initial = db.get_active_symbols().await.unwrap();
        assert!(initial.is_empty(), "Should have no active symbols initially");

        // Add some price data
        let price_infos = vec![
            PriceInfo {
                symbol: "BTCUSDT".to_string(),
                price: 50000.0,
                price_change_percent: 2.5,
                volume: 1000.0,
                high_24h: 51000.0,
                low_24h: 49000.0,
                prev_close_price: 48750.0,
            },
            PriceInfo {
                symbol: "ETHUSDT".to_string(),
                price: 3000.0,
                price_change_percent: -1.2,
                volume: 500.0,
                high_24h: 3100.0,
                low_24h: 2900.0,
                prev_close_price: 3036.0,
            },
        ];

        db.store_price_infos(&price_infos).await.unwrap();

        // Should return active symbols
        let active = db.get_active_symbols().await.unwrap();
        assert_eq!(active.len(), 2, "Should return both symbols");
        assert!(active.contains(&"BTCUSDT".to_string()));
        assert!(active.contains(&"ETHUSDT".to_string()));
    });
}

#[test]
fn test_nonexistent_symbol_returns_none() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    block_on(async {
        let db = Database::new(db_path).await.unwrap();

        let result = db.get_latest_price("NONEXISTENT").await.unwrap();
        assert!(result.is_none(), "Should return None for nonexistent symbol");
    });
}

#[test]
fn test_empty_candle_storage() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    block_on(async {
        let db = Database::new(db_path).await.unwrap();

        // Should handle empty candle array gracefully
        let result = db.store_candles("BTCUSDT", "1m", &[]).await;
        assert!(result.is_ok(), "Should handle empty candle array");

        let retrieved = db.get_candles("BTCUSDT", "1m", 10).await.unwrap();
        assert!(retrieved.is_empty(), "Should return empty array");
    });
}
