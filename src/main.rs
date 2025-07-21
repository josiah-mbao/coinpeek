mod app;
mod binance;
mod input;
mod theme;
mod ui;
mod utils;

use std::error::Error;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Style, Stylize},
        text::{Span, Line, Text},
        widgets::{Block, Borders, Paragraph},
    };

    use crossterm::event::{self, Event, KeyCode};
    use std::time::{Duration, Instant};

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(10);
    let mut price = binance::fetch_price("BTCUSDT").await?;

    loop {
        terminal.draw(|f| {
            let size = f.area(); // ✅ replaces deprecated f.size()

            let block = Block::default()
                .title("🚀 CoinPeek")
                .borders(Borders::ALL);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(size);

            let price_text = Paragraph::new(Text::from(Line::from(vec![
                Span::raw("BTC/USDT: "),
                Span::styled(format!("${:.2}", price), Style::default().bold()),
            ])));

            f.render_widget(block, size);
            f.render_widget(price_text, layout[0]);
        })?;

        let timeout = Duration::from_millis(200);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let new_price = binance::fetch_price("BTCUSDT").await?;
            if (new_price - price).abs() > 0.1 {
                price = new_price;
            }
            last_tick = Instant::now();
        }
    }

    Ok(())
}
