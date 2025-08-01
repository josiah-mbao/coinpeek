mod app;
mod binance;
mod input;
mod theme;
mod ui;
mod utils;

use std::error::Error;
use std::io;

use crossterm::event::{EnableMouseCapture, DisableMouseCapture};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = init_terminal()?;
    let result = run_loop(&mut terminal).await;
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
) -> Result<(), Box<dyn Error>> {
    let symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT", "DOGEUSDT"];
    let mut prices = binance::fetch_prices(&symbols).await;

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(2);

    loop {
        terminal.draw(|f| {
            let size = f.size();
            ui::render_dashboard(f, size, &prices);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            prices = binance::fetch_prices(&symbols).await;
            last_tick = Instant::now();
        }
    }

    Ok(())
}
