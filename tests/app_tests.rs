use coinpeek::app::{App, SortMode};
use coinpeek::config::Config;
use coinpeek::binance::{PriceInfo, Candle};

#[test]
fn test_app_initialization() {
    let config = Config {
        symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        refresh_interval_seconds: 30,
    };

    let app = App::new(config);

    assert!(app.price_infos.is_empty());
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.sort_mode, SortMode::Symbol);
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

    // Test price sorting (highest price first)
    app.next_sort_mode();
    assert_eq!(app.sort_mode, SortMode::Price);
    app.update_prices(price_infos.clone()); // Re-sort
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT"); // Higher price first (50000 > 3000)
    assert_eq!(app.price_infos[1].symbol, "ETHUSDT");

    // Test change percent sorting (highest change percent first)
    app.next_sort_mode();
    assert_eq!(app.sort_mode, SortMode::ChangePercent);
    app.update_prices(price_infos.clone());
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT"); // Higher change percent first (+2.5% > -1.2%)
    assert_eq!(app.price_infos[1].symbol, "ETHUSDT");

    // Test volume sorting (highest volume first)
    app.next_sort_mode();
    assert_eq!(app.sort_mode, SortMode::Volume);
    app.update_prices(price_infos.clone());
    assert_eq!(app.price_infos[0].symbol, "BTCUSDT"); // Higher volume first (1000 > 500)
    assert_eq!(app.price_infos[1].symbol, "ETHUSDT");

    // Cycle back to symbol sorting
    app.next_sort_mode();
    assert_eq!(app.sort_mode, SortMode::Symbol);
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
