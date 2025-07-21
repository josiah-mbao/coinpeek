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
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

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
) -> Result<(), Box<dyn Error>> {
    // Placeholder TUI render loop
    use std::time::{Duration, Instant};

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = ratatui::widgets::Block::default()
                .title("🚀 coinpeek")
                .borders(ratatui::widgets::Borders::ALL);
            f.render_widget(block, size);
        })?;

        // simple break on key press
        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == event::KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}
