// src/ui.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

/// Draws the main crypto dashboard UI
pub fn render_dashboard(
    f: &mut Frame,
    area: Rect,
    prices: &[(String, f64)],
) {
    let block = Block::default()
        .title("🚀 CoinPeek")
        .borders(Borders::ALL);

    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            std::iter::repeat(Constraint::Length(1))
                .take(prices.len())
                .collect::<Vec<_>>(),
        )
        .split(inner_area);

    for (i, (symbol, price)) in prices.iter().enumerate() {
        let line = Line::from(vec![
            Span::raw(format!("{:<8}: ", symbol)),
            Span::styled(format!("${:.2}", price), Style::default().bold()),
        ]);

        let widget = Paragraph::new(Text::from(line));
        f.render_widget(widget, layout[i]);
    }
}
