mod app;
mod binance;
mod config;
mod database;
mod input;
mod theme;
mod ui;
mod utils;

use std::error::Error;
use std::io;

use crossterm::event::{EnableMouseCapture, DisableMouseCapture, MouseEvent, MouseEventKind};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::time::{Duration, Instant};

/// Handle mouse click events for cryptocurrency selection
fn handle_mouse_click(app: &mut app::App, mouse_event: MouseEvent) {
    // Only handle left mouse button down events
    if mouse_event.kind != MouseEventKind::Down(crossterm::event::MouseButton::Left) {
        return;
    }

    let mouse_x = mouse_event.column as usize;
    let mouse_y = mouse_event.row as usize;

    // Account for main block borders (1 line/column offset)
    // Mouse coordinates are 0-based from terminal top-left
    if mouse_x < 1 || mouse_y < 1 {
        return; // Clicked in border area
    }

    let _inner_x = mouse_x - 1;
    let inner_y = mouse_y - 1;

    // Note: X coordinate checking would require terminal width knowledge
    // For now, we accept clicks anywhere and assume they're in the left panel
    // In a more sophisticated implementation, we'd pass terminal dimensions

    // Skip if no cryptocurrencies to select
    if app.price_infos.is_empty() {
        return;
    }

    // Approximate layout:
    // - Title bar: 1 line
    // - Top margin: 1 line
    // - Each crypto: 3 lines
    // So crypto at index i starts at Y = 1 + 1 + (i * 3) = 2 + (i * 3)

    let title_height = 1;
    let top_margin = 1;
    let crypto_height = 3;

    // Calculate which crypto was clicked
    let crypto_start_y = title_height + top_margin;
    if inner_y < crypto_start_y {
        return; // Clicked in title or margin area
    }

    let relative_y = inner_y - crypto_start_y;
    let clicked_index = relative_y / crypto_height;

    // Bounds check
    if clicked_index < app.price_infos.len() {
        app.selected_index = clicked_index;
        // Note: candle fetching will happen in the main loop
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration
    let config = config::Config::load()?;

    let mut terminal = init_terminal()?;
    let result = run_loop(&mut terminal, config).await;
    cleanup_terminal(&mut terminal)?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

/// Initializes the terminal in raw mode with alternate screen and mouse capture
fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restores the terminal to normal mode
fn cleanup_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Main application loop
async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db = database::Database::new("coinpeek.db").await?;
    println!("Database initialized successfully");

    let symbols: Vec<&str> = config.symbols.iter().map(|s| s.as_str()).collect();
    let mut app = app::App::new(config.clone());

    // Try to load cached price data first
    let mut cached_prices = Vec::new();
    for symbol in &symbols {
        if let Ok(Some(price_info)) = db.get_latest_price(symbol).await {
            cached_prices.push(price_info);
        }
    }

    if !cached_prices.is_empty() {
        println!("Loaded {} cached prices", cached_prices.len());
        app.update_prices(cached_prices);
    }

    // Initial API fetch for fresh data
    if let Ok(price_infos) = binance::fetch_price_infos(&symbols).await {
        // Store in database
        db.store_price_infos(&price_infos).await?;
        app.record_successful_sync();
        app.update_prices(price_infos);
    } else {
        app.record_sync_failure();
    }

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(config.refresh_interval_seconds);

    loop {
        terminal.draw(|f| {
            let size = f.area();
            ui::render_dashboard(f, size, &app);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // If help is showing, any key closes it
                if app.show_help {
                    app.toggle_help();
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => app.select_previous(),
                    KeyCode::Down => app.select_next(),
                    KeyCode::Char('s') => app.next_sort_mode(),
                    KeyCode::Char('d') => app.toggle_sort_direction(),
                    KeyCode::Char('f') => app.next_filter_preset(),
                    KeyCode::Char('c') => app.clear_all_filters(),
                    KeyCode::Char('o') => app.toggle_offline_mode(),
                    KeyCode::Char('p') => app.toggle_pause(),
                    KeyCode::Char('?') => app.toggle_help(),
                    KeyCode::Char('r') => {
                        // Manual refresh
                        if let Ok(price_infos) = binance::fetch_price_infos(&symbols).await {
                            // Store in database
                            if let Err(e) = db.store_price_infos(&price_infos).await {
                                eprintln!("Failed to store prices: {}", e);
                            }
                            app.record_successful_sync();
                            app.update_prices(price_infos);
                        } else {
                            app.record_sync_failure();
                        }
                    }
                    _ => {}
                }
            } else if let Event::Mouse(mouse_event) = event::read()? {
                // Handle mouse events when help is not active
                if !app.show_help {
                    handle_mouse_click(&mut app, mouse_event);
                }
            }
        }

        if !app.paused && last_tick.elapsed() >= tick_rate {
            if let Ok(price_infos) = binance::fetch_price_infos(&symbols).await {
                // Store in database
                if let Err(e) = db.store_price_infos(&price_infos).await {
                    eprintln!("Failed to store prices: {}", e);
                }
                app.record_successful_sync();
                app.update_prices(price_infos);
            } else {
                app.record_sync_failure();
            }
            last_tick = Instant::now();
        }

        // Fetch candle data for selected symbol if needed
        if let Some(symbol) = app.should_fetch_candles() {
            // Try to load from cache first
            if let Ok(cached_candles) = db.get_candles(&symbol, "5m", 50).await {
                if !cached_candles.is_empty() {
                    app.update_candles_for_selected(cached_candles);
                } else {
                    // Fetch from API if not in cache
                    if let Ok(candles) = binance::fetch_candles(&symbol, "5m", 50).await {
                        // Store in database
                        if let Err(e) = db.store_candles(&symbol, "5m", &candles).await {
                            eprintln!("Failed to store candles: {}", e);
                        }
                        app.update_candles_for_selected(candles);
                    }
                }
            } else {
                // Fallback to API if database query fails
                if let Ok(candles) = binance::fetch_candles(&symbol, "5m", 50).await {
                    // Store in database
                    if let Err(e) = db.store_candles(&symbol, "5m", &candles).await {
                        eprintln!("Failed to store candles: {}", e);
                    }
                    app.update_candles_for_selected(candles);
                }
            }
        }
    }

    Ok(())
}
