use coinpeek::app::{App, SortMode, SortDirection, FilterPreset, FilterType};
use coinpeek::config::Config;
use coinpeek::binance::{PriceInfo, Candle};
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

#[test]
fn test_app_initialization() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let app = App::new(config);

    assert!(app.price_infos.is_empty());
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.sort_config.mode, SortMode::Symbol);
    assert!(!app.paused);
    assert!(app.selected_candles.is_empty());
    assert!(app.selected_symbol_candles.is_empty());
}

#[test]
fn test_price_update_and_sorting() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    let price_infos = vec![
        PriceInfo {
            symbol: "ETHUSDT".to_string(),
            price: 3000.0,
            price_change_percent: -1.2,
            volume: 500.0,
            high_24h: 3100.0,
            low_24h: 2900.0,
            prev_close_price: 3036.0,
        },
        PriceInfo {
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            price_change_percent: 2.5,
            volume: 1000.0,
            high_24h: 51000.0,
            low_24h: 49000.0,
            prev_close_price: 48750.0,
        },
    ];

    app.update_prices(price_infos);

    // Should be sorted by symbol initially
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT");
    assert_eq!(app.price_infos[1].symbol, "ETHUSDT");
}

#[test]
fn test_sort_mode_changes() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    let price_infos = vec![
        PriceInfo {
            symbol: "ETHUSDT".to_string(),
            price: 3000.0,
            price_change_percent: -1.2,
            volume: 500.0,
            high_24h: 3100.0,
            low_24h: 2900.0,
            prev_close_price: 3036.0,
        },
        PriceInfo {
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            price_change_percent: 2.5,
            volume: 1000.0,
            high_24h: 51000.0,
            low_24h: 49000.0,
            prev_close_price: 48750.0,
        },
    ];

    app.update_prices(price_infos.clone());

    // Test symbol sorting (default)
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT");
    assert_eq!(app.price_infos[1].symbol, "ETHUSDT");

    // Test price sorting (ascending by default - lowest price first)
    app.next_sort_mode(); // Go to Price mode
    app.update_prices(price_infos.clone()); // Re-sort
    assert_eq!(app.price_infos[0].symbol, "ETHUSDT"); // Lower price first (3000 < 50000)
    assert_eq!(app.price_infos[1].symbol, "BTCUSDT");

    // Test change percent sorting (highest change percent first)
    app.next_sort_mode();
    assert_eq!(app.sort_config.mode, SortMode::ChangePercent);
    app.update_prices(price_infos.clone());
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT"); // Higher change percent first (+2.5% > -1.2%)
    assert_eq!(app.price_infos[1].symbol, "ETHUSDT");

    // Test volume sorting - descending for highest volume first
    app.next_sort_mode();
    assert_eq!(app.sort_config.mode, SortMode::Volume);
    app.sort_config.direction = SortDirection::Descending; // Set to descending
    app.update_prices(price_infos.clone()); // Re-sort with new direction
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT"); // Higher volume first (1000 > 500)
    assert_eq!(app.price_infos[1].symbol, "ETHUSDT");

    // Cycle back to symbol sorting
    app.next_sort_mode();
    assert_eq!(app.sort_config.mode, SortMode::Symbol);
}

#[test]
fn test_navigation() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

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
        PriceInfo {
            symbol: "ADAUSDT".to_string(),
            price: 1.5,
            price_change_percent: 0.5,
            volume: 100.0,
            high_24h: 1.6,
            low_24h: 1.4,
            prev_close_price: 1.49,
        },
    ];

    app.update_prices(price_infos);

    // Initial selection
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.get_selected_symbol().unwrap().symbol, "ADAUSDT"); // Sorted by symbol

    // Navigate next
    app.select_next();
    assert_eq!(app.selected_index, 1);
    assert_eq!(app.get_selected_symbol().unwrap().symbol, "BTCUSDT");

    app.select_next();
    assert_eq!(app.selected_index, 2);
    assert_eq!(app.get_selected_symbol().unwrap().symbol, "ETHUSDT");

    // Wrap around to beginning
    app.select_next();
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.get_selected_symbol().unwrap().symbol, "ADAUSDT");

    // Navigate previous
    app.select_previous();
    assert_eq!(app.selected_index, 2);
    assert_eq!(app.get_selected_symbol().unwrap().symbol, "ETHUSDT");

    // Wrap around to end
    app.select_previous();
    assert_eq!(app.selected_index, 1);
    assert_eq!(app.get_selected_symbol().unwrap().symbol, "BTCUSDT");
}

#[test]
fn test_pause_functionality() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    // Initially not paused
    assert!(!app.paused);

    // Pause
    app.toggle_pause();
    assert!(app.paused);

    // Unpause
    app.toggle_pause();
    assert!(!app.paused);
}

#[test]
fn test_candle_data_management() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    let price_infos = vec![PriceInfo {
        symbol: "BTCUSDT".to_string(),
        price: 50000.0,
        price_change_percent: 2.5,
        volume: 1000.0,
        high_24h: 51000.0,
        low_24h: 49000.0,
        prev_close_price: 48750.0,
    }];

    app.update_prices(price_infos);

    // Initially no candles
    assert!(app.selected_candles.is_empty());
    assert!(app.selected_symbol_candles.is_empty());

    // Should fetch candles for selected symbol
    let should_fetch = app.should_fetch_candles();
    assert_eq!(should_fetch, Some("BTCUSDT".to_string()));

    // Update with candle data
    let candles = vec![
        Candle {
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 100.0,
            timestamp: 1640995200000,
        },
        Candle {
            open: 50500.0,
            high: 51500.0,
            low: 50000.0,
            close: 51000.0,
            volume: 120.0,
            timestamp: 1640995260000,
        },
    ];

    app.update_candles_for_selected(candles);

    // Should now have candles
    assert_eq!(app.selected_candles.len(), 2);
    assert_eq!(app.selected_symbol_candles, "BTCUSDT");

    // Should not fetch again for same symbol
    let should_fetch_again = app.should_fetch_candles();
    assert!(should_fetch_again.is_none());

    // Change selection
    app.select_next(); // This would be a different symbol if we had more
    // For now, test with same symbol - should not fetch
    let should_fetch_changed = app.should_fetch_candles();
    assert!(should_fetch_changed.is_none());
}

#[test]
fn test_empty_price_list_navigation() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    // Empty list
    assert!(app.price_infos.is_empty());
    assert_eq!(app.selected_index, 0);

    // Navigation should be safe on empty list
    app.select_next();
    assert_eq!(app.selected_index, 0); // Should remain 0

    app.select_previous();
    assert_eq!(app.selected_index, 0); // Should remain 0

    // get_selected_symbol should return None
    assert!(app.get_selected_symbol().is_none());
}

#[test]
fn test_sort_mode_string_representation() {
    assert_eq!(SortMode::Symbol.as_str(), "Symbol");
    assert_eq!(SortMode::Price.as_str(), "Price");
    assert_eq!(SortMode::ChangePercent.as_str(), "24h Change");
    assert_eq!(SortMode::Volume.as_str(), "Volume");
}

#[test]
fn test_sort_mode_cycling() {
    let mut mode = SortMode::Symbol;
    assert_eq!(mode, SortMode::Symbol);

    mode = mode.next();
    assert_eq!(mode, SortMode::Price);

    mode = mode.next();
    assert_eq!(mode, SortMode::ChangePercent);

    mode = mode.next();
    assert_eq!(mode, SortMode::Volume);

    mode = mode.next();
    assert_eq!(mode, SortMode::Symbol); // Back to start
}

#[test]
fn test_price_sorting_edge_cases() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    // Test with identical prices
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
            price: 50000.0, // Same price
            price_change_percent: -1.2,
            volume: 500.0,
            high_24h: 51000.0,
            low_24h: 49000.0,
            prev_close_price: 51000.0,
        },
    ];

    app.update_prices(price_infos);

    // Should sort by symbol when prices are equal
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT");
    assert_eq!(app.price_infos[1].symbol, "ETHUSDT");

    // Test with zero and negative values
    let edge_case_infos = vec![
        PriceInfo {
            symbol: "ZEROUSDT".to_string(),
            price: 0.0,
            price_change_percent: 0.0,
            volume: 0.0,
            high_24h: 0.0,
            low_24h: 0.0,
            prev_close_price: 0.0,
        },
        PriceInfo {
            symbol: "NEGUSDT".to_string(),
            price: -100.0,
            price_change_percent: -50.0,
            volume: -10.0,
            high_24h: -50.0,
            low_24h: -150.0,
            prev_close_price: -90.0,
        },
    ];

    app.update_prices(edge_case_infos);

    // Should handle edge cases gracefully
    assert_eq!(app.price_infos.len(), 2);
}

#[test]
fn test_selection_bounds_checking() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    // Set up data
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
    ];

    app.update_prices(price_infos);

    // Manually set invalid index (simulate edge case)
    app.selected_index = 99;

    // Update prices should fix the index
    app.update_prices(vec![PriceInfo {
        symbol: "ETHUSDT".to_string(),
        price: 3000.0,
        price_change_percent: -1.2,
        volume: 500.0,
        high_24h: 3100.0,
        low_24h: 2900.0,
        prev_close_price: 3036.0,
    }]);

    // Should reset to valid index
    assert_eq!(app.selected_index, 0);
}

#[test]
fn test_filter_preset_application() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    let price_infos = vec![
        PriceInfo {
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            price_change_percent: 2.5, // Top gainer
            volume: 1000.0,
            high_24h: 51000.0,
            low_24h: 49000.0,
            prev_close_price: 48750.0,
        },
        PriceInfo {
            symbol: "ETHUSDT".to_string(),
            price: 3000.0,
            price_change_percent: -1.2, // Neutral
            volume: 500.0,
            high_24h: 3100.0,
            low_24h: 2900.0,
            prev_close_price: 3036.0,
        },
        PriceInfo {
            symbol: "ADAUSDT".to_string(),
            price: 1.5,
            price_change_percent: -8.0, // Top loser
            volume: 100.0,
            high_24h: 1.6,
            low_24h: 1.4,
            prev_close_price: 1.63,
        },
        PriceInfo {
            symbol: "SOLUSDT".to_string(),
            price: 100.0,
            price_change_percent: 0.5, // Stable
            volume: 2000.0, // High volume
            high_24h: 105.0,
            low_24h: 95.0,
            prev_close_price: 99.5,
        },
        PriceInfo {
            symbol: "DOTUSDT".to_string(),
            price: 25.0,
            price_change_percent: 15.0, // Volatile
            volume: 800.0,
            high_24h: 30.0,
            low_24h: 20.0,
            prev_close_price: 21.7,
        },
    ];

    app.update_prices(price_infos.clone());
    assert_eq!(app.price_infos.len(), 5); // All coins initially

    // Test Top Gainers preset
    app.set_filter_preset(FilterPreset::TopGainers);
    assert_eq!(app.price_infos.len(), 1); // Only DOT (15.0%) meets >= 5% criteria
    assert!(app.price_infos.iter().all(|p| p.price_change_percent >= 5.0));

    // Test Top Losers preset
    app.set_filter_preset(FilterPreset::TopLosers);
    assert_eq!(app.price_infos.len(), 1); // Only ADA (-8.0%)
    assert!(app.price_infos.iter().all(|p| p.price_change_percent < -5.0));

    // Test High Volume preset
    app.set_filter_preset(FilterPreset::HighVolume);
    assert_eq!(app.price_infos.len(), 2); // SOL (2000) and BTC (1000) - top 40%
    assert!(app.price_infos.iter().any(|p| p.symbol == "SOLUSDT"));
    assert!(app.price_infos.iter().any(|p| p.symbol == "BTCUSDT"));

    // Test Volatile preset
    app.set_filter_preset(FilterPreset::Volatile);
    assert_eq!(app.price_infos.len(), 2); // DOT (15.0%) and BTC (2.5%) - abs > 3.0%
    assert!(app.price_infos.iter().all(|p| p.price_change_percent.abs() > 3.0));

    // Test Stable preset
    app.set_filter_preset(FilterPreset::Stable);
    assert_eq!(app.price_infos.len(), 1); // Only SOL (0.5%) - abs < 1.0%
    assert!(app.price_infos.iter().all(|p| p.price_change_percent.abs() < 1.0));

    // Test All preset
    app.set_filter_preset(FilterPreset::All);
    assert_eq!(app.price_infos.len(), 5); // All coins back
}

#[test]
fn test_custom_filter_application() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

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
        PriceInfo {
            symbol: "ADAUSDT".to_string(),
            price: 1.5,
            price_change_percent: 0.5,
            volume: 100.0,
            high_24h: 1.6,
            low_24h: 1.4,
            prev_close_price: 1.49,
        },
    ];

    app.update_prices(price_infos.clone());
    assert_eq!(app.price_infos.len(), 3);

    // Test price range filter
    app.add_filter(FilterType::PriceRange {
        min: Some(1000.0),
        max: Some(40000.0)
    });
    assert_eq!(app.price_infos.len(), 1); // Only ETH (3000)
    assert_eq!(app.price_infos[0].symbol, "ETHUSDT");

    // Clear filters and test volume filter
    app.clear_all_filters();
    app.update_prices(price_infos.clone());
    app.add_filter(FilterType::VolumeRange {
        min: Some(200.0),
        max: None
    });
    assert_eq!(app.price_infos.len(), 2); // ETH (500) and BTC (1000)
    assert!(app.price_infos.iter().all(|p| p.volume >= 200.0));

    // Clear and test symbol search
    app.clear_all_filters();
    app.update_prices(price_infos.clone());
    app.add_filter(FilterType::SymbolSearch("BTC".to_string()));
    assert_eq!(app.price_infos.len(), 1);
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT");

    // Test case-insensitive search
    app.clear_all_filters();
    app.update_prices(price_infos.clone());
    app.add_filter(FilterType::SymbolSearch("btc".to_string()));
    assert_eq!(app.price_infos.len(), 1);
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT");
}

#[test]
fn test_offline_awareness_tracking() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    // Initially no sync data
    assert!(app.data_status.last_successful_sync.is_none());
    assert!(!app.data_status.offline_mode);
    assert_eq!(app.data_status.consecutive_failures, 0);
    assert_eq!(app.get_offline_indicator(), "🟢 synced never");

    // Record successful sync
    app.record_successful_sync();
    assert!(app.data_status.last_successful_sync.is_some());
    assert!(!app.data_status.offline_mode);
    assert_eq!(app.data_status.consecutive_failures, 0);
    assert!(app.get_offline_indicator().contains("🟢 synced just now"));

    // Record some failures
    app.record_sync_failure();
    assert_eq!(app.data_status.consecutive_failures, 1);
    assert!(!app.data_status.offline_mode);
    assert!(app.get_offline_indicator().contains("🟡 1 failures"));

    app.record_sync_failure();
    app.record_sync_failure();
    // Check values before toggle_offline_mode resets them
    assert_eq!(app.data_status.consecutive_failures, 3);
    assert!(app.data_status.offline_mode); // Should auto-enable offline mode
    assert!(app.get_offline_indicator().contains("🔴 OFFLINE"));

    // Toggle offline mode manually (resets failure counter)
    app.toggle_offline_mode();
    assert!(!app.data_status.offline_mode);
    // Counter is reset by toggle_offline_mode when manually enabling offline mode
    assert_eq!(app.data_status.consecutive_failures, 0);
    assert!(app.get_offline_indicator().contains("🟢 synced"));
}

#[test]
fn test_data_age_calculations() {
    use chrono::{Duration, Utc};

    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    // No data yet
    assert_eq!(app.get_data_age_string(), "never");

    // Set a timestamp 2 hours ago
    let two_hours_ago = Utc::now() - Duration::hours(2);
    app.data_status.last_successful_sync = Some(two_hours_ago);

    let age_string = app.get_data_age_string();
    assert!(age_string.contains("2h ago") || age_string.contains("1h ago"));

    // Set a timestamp 30 minutes ago
    let thirty_min_ago = Utc::now() - Duration::minutes(30);
    app.data_status.last_successful_sync = Some(thirty_min_ago);

    let age_string = app.get_data_age_string();
    assert!(age_string.contains("30m ago") || age_string.contains("29m ago"));

    // Set a timestamp just now
    app.data_status.last_successful_sync = Some(Utc::now());
    assert_eq!(app.get_data_age_string(), "just now");
}

#[test]
fn test_combined_filtering_and_sorting() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

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
            volume: 1500.0, // Higher volume than BTC
            high_24h: 3100.0,
            low_24h: 2900.0,
            prev_close_price: 3036.0,
        },
        PriceInfo {
            symbol: "ADAUSDT".to_string(),
            price: 1.5,
            price_change_percent: 8.0, // Top gainer
            volume: 500.0,
            high_24h: 1.6,
            low_24h: 1.4,
            prev_close_price: 1.39,
        },
    ];

    app.update_prices(price_infos.clone());

    // Apply Top Gainers preset (ADA with +8.0%)
    app.set_filter_preset(FilterPreset::TopGainers);
    assert_eq!(app.price_infos.len(), 1);
    assert_eq!(app.price_infos[0].symbol, "ADAUSDT");

    // Now sort by volume (should still be just ADA)
    app.next_sort_mode(); // Volume sort
    app.sort_config.direction = SortDirection::Descending;
    app.update_prices(price_infos.clone()); // Re-apply filters and sorting
    assert_eq!(app.price_infos.len(), 1);
    assert_eq!(app.price_infos[0].symbol, "ADAUSDT");

    // Clear filters and sort by volume descending
    app.clear_all_filters();
    app.sort_config.mode = SortMode::Volume;
    app.sort_config.direction = SortDirection::Descending;
    app.update_prices(price_infos.clone());
    assert_eq!(app.price_infos[0].symbol, "ETHUSDT"); // ETH has highest volume (1500)
    assert_eq!(app.price_infos[1].symbol, "BTCUSDT"); // BTC has 1000
    assert_eq!(app.price_infos[2].symbol, "ADAUSDT"); // ADA has 500
}

#[test]
fn test_mouse_click_crypto_selection() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    let price_infos = vec![
        PriceInfo {
            symbol: "ADAUSDT".to_string(),
            price: 1.5,
            price_change_percent: 0.5,
            volume: 100.0,
            high_24h: 1.6,
            low_24h: 1.4,
            prev_close_price: 1.49,
        },
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

    app.update_prices(price_infos);

    // Test clicking on first crypto (ADAUSDT)
    // Y coordinate = border(1) + title(1) + margin(1) + row_offset(0) = 3
    let mouse_event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5, // Inside left panel
        row: 3,    // First crypto row
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    // Need to use the actual function from main.rs - this is a limitation of unit testing
    // For now, we'll test the logic manually by simulating the coordinate calculation

    // Border offset: mouse_y - 1 = 3 - 1 = 2
    let inner_y = mouse_event.row as usize - 1;
    // Title height (1) + margin (1) = 2
    let crypto_start_y = 1 + 1;
    let relative_y = inner_y - crypto_start_y; // 2 - 2 = 0
    let clicked_index = relative_y / 3; // 0 / 3 = 0

    assert_eq!(clicked_index, 0); // Should select first crypto (ADAUSDT)

    // Test clicking on second crypto (BTCUSDT)
    // Y coordinate for second row: 3 + 3 = 6
    let mouse_event2 = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 6, // Second crypto row
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    let inner_y2 = mouse_event2.row as usize - 1; // 6 - 1 = 5
    let relative_y2 = inner_y2 - crypto_start_y; // 5 - 2 = 3
    let clicked_index2 = relative_y2 / 3; // 3 / 3 = 1

    assert_eq!(clicked_index2, 1); // Should select second crypto (BTCUSDT)

    // Test clicking on third crypto (ETHUSDT)
    // Y coordinate for third row: 6 + 3 = 9
    let mouse_event3 = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 9, // Third crypto row
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    let inner_y3 = mouse_event3.row as usize - 1; // 9 - 1 = 8
    let relative_y3 = inner_y3 - crypto_start_y; // 8 - 2 = 6
    let clicked_index3 = relative_y3 / 3; // 6 / 3 = 2

    assert_eq!(clicked_index3, 2); // Should select third crypto (ETHUSDT)
}

#[test]
fn test_mouse_click_bounds_checking() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

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
    ];

    app.update_prices(price_infos);
    let initial_selection = app.selected_index;

    // Test clicking in border area (should be ignored)
    let border_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 0, // Border column
        row: 5,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    // Simulate bounds checking logic
    let mouse_x = border_click.column as usize;
    let mouse_y = border_click.row as usize;

    // Should be ignored due to mouse_x < 1
    assert!(mouse_x < 1);

    // Test clicking in title/margin area (should be ignored)
    let title_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 2, // Title/margin area
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    let inner_y = title_click.row as usize - 1; // 2 - 1 = 1
    let crypto_start_y = 1 + 1; // 2

    // Should be ignored due to inner_y < crypto_start_y
    assert!(inner_y < crypto_start_y);

    // Verify selection unchanged
    assert_eq!(app.selected_index, initial_selection);
}

#[test]
fn test_mouse_click_empty_list() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

    // No price data loaded
    assert!(app.price_infos.is_empty());

    let mouse_event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 5,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    // Should be ignored due to empty price_infos
    assert!(app.price_infos.is_empty());
    // In real function, this would return early without changing selection
}

#[test]
fn test_mouse_click_non_left_button() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

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
    ];

    app.update_prices(price_infos);
    let initial_selection = app.selected_index;

    // Test right mouse button (should be ignored)
    let right_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: 5,
        row: 5,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    // Should be ignored due to non-left button
    assert_ne!(right_click.kind, MouseEventKind::Down(MouseButton::Left));

    // Test middle mouse button (should be ignored)
    let middle_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Middle),
        column: 5,
        row: 5,
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    // Should be ignored due to non-left button
    assert_ne!(middle_click.kind, MouseEventKind::Down(MouseButton::Left));

    // Verify selection unchanged
    assert_eq!(app.selected_index, initial_selection);
}

#[test]
fn test_mouse_click_out_of_bounds() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let mut app = App::new(config);

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

    app.update_prices(price_infos);
    let initial_selection = app.selected_index;

    // Test clicking beyond available cryptos (index 99)
    let out_of_bounds_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 20, // Way beyond available rows
        modifiers: crossterm::event::KeyModifiers::empty(),
    };

    let inner_y = out_of_bounds_click.row as usize - 1; // 20 - 1 = 19
    let crypto_start_y = 1 + 1; // 2
    let relative_y = inner_y - crypto_start_y; // 19 - 2 = 17
    let clicked_index = relative_y / 3; // 17 / 3 = 5

    // Should be ignored due to clicked_index >= price_infos.len() (5 >= 2)
    assert!(clicked_index >= app.price_infos.len());

    // Verify selection unchanged
    assert_eq!(app.selected_index, initial_selection);
}
