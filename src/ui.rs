// src/ui.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::app::App;

/// Draws the main crypto dashboard UI
pub fn render_dashboard(
    f: &mut Frame,
    area: Rect,
    app: &App,
) {
    // If help is active, show only the help screen (clear the dashboard)
    if app.show_help {
        render_help_screen(f, area);
        return;
    }

    let main_block = Block::default()
        .title("🚀 CoinPeek")
        .borders(Borders::ALL);

    f.render_widget(main_block.clone(), area);

    let main_area = main_block.inner(area);

    // Split into left (list) and right (details) panels
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left panel: crypto list
            Constraint::Percentage(50), // Right panel: detailed view
        ])
        .split(main_area);

    // Left panel: Crypto list
    render_crypto_list(f, main_layout[0], app);

    // Right panel: Detailed view of selected crypto
    render_crypto_details(f, main_layout[1], app);
}

fn render_crypto_list(f: &mut Frame, area: Rect, app: &App) {
    // Create title with sort, filter, and sync status info
    let sort_info = app.sort_config.display_name();
    let filter_info = app.get_filter_status();
    let (visible, total) = app.get_visible_count();
    let sync_status = app.get_offline_indicator();

    let base_title = format!("📊 Cryptocurrency Prices | {} | {} | {}/{} coins | {}",
                           sort_info, filter_info, visible, total, sync_status);

    let title_with_search = if app.search_mode {
        format!("🔍 Search: \"{}\" | {}", app.search_query, base_title)
    } else {
        base_title
    };

    // Add error status to title if there are active errors
    let title = if let Some(error_summary) = app.get_error_summary() {
        format!("{} | {}", error_summary, title_with_search)
    } else {
        title_with_search
    };

    let list_block = Block::default()
        .title(title)
        .borders(Borders::ALL);

    f.render_widget(list_block.clone(), area);

    let list_area = list_block.inner(area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            std::iter::repeat(Constraint::Length(3))
                .take(app.price_infos.len())
                .collect::<Vec<_>>(),
        )
        .split(list_area);

    for (i, price_info) in app.price_infos.iter().enumerate() {
        let is_selected = i == app.selected_index;

        let change_color = if price_info.price_change_percent > 0.0 {
            Color::Green
        } else if price_info.price_change_percent < 0.0 {
            Color::Red
        } else {
            Color::Gray
        };

        let change_symbol = if price_info.price_change_percent > 0.0 {
            "▲"
        } else if price_info.price_change_percent < 0.0 {
            "▼"
        } else {
            "■"
        };

        let mut price_line = vec![
            Span::raw("  "), // Add some left padding
            Span::raw(format!("{:<8}: ", price_info.symbol)),
            Span::styled(format!("${:.2}", price_info.price), Style::default().bold()),
            Span::raw("  "), // Add spacing before change
            Span::styled(
                format!("{} {:.2}%", change_symbol, price_info.price_change_percent),
                Style::default().fg(change_color),
            ),
        ];

        let mut volume_line = vec![
            Span::raw("           "), // Add left padding to align with price line
            Span::styled(
                format!("Vol: {:.0}", price_info.volume),
                Style::default().fg(Color::Blue),
            ),
            Span::raw("  "), // Add spacing
            Span::styled(
                format!("H:{:.2} L:{:.2}", price_info.high_24h, price_info.low_24h),
                Style::default().fg(Color::Gray),
            ),
        ];

        // Apply selection highlighting
        let bg_color = if is_selected {
            Color::Blue
        } else {
            Color::Reset
        };

        for span in &mut price_line {
            span.style = span.style.bg(bg_color);
        }
        for span in &mut volume_line {
            span.style = span.style.bg(bg_color);
        }

        let price_line = Line::from(price_line);
        let volume_line = Line::from(volume_line);

        let text = Text::from(vec![price_line, volume_line]);
        let widget = Paragraph::new(text);
        f.render_widget(widget, layout[i]);
    }
}

fn render_crypto_details(f: &mut Frame, area: Rect, app: &App) {
    let details_block = Block::default()
        .title("🔍 Detailed View")
        .borders(Borders::ALL);

    f.render_widget(details_block.clone(), area);

    let details_area = details_block.inner(area);

    if let Some(selected_crypto) = app.price_infos.get(app.selected_index) {
        let details_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Symbol and name
                Constraint::Length(4), // Current price (large)
                Constraint::Length(6), // Price chart (sparkline)
                Constraint::Length(2), // 24h change
                Constraint::Length(2), // 24h high/low
                Constraint::Length(2), // Volume
            ])
            .split(details_area);

        // Symbol and name (mock full name for now)
        let symbol_name = match selected_crypto.symbol.as_str() {
            "BTCUSDT" => "Bitcoin",
            "ETHUSDT" => "Ethereum",
            "BNBUSDT" => "Binance Coin",
            "ADAUSDT" => "Cardano",
            "SOLUSDT" => "Solana",
            "DOTUSDT" => "Polkadot",
            "DOGEUSDT" => "Dogecoin",
            "AVAXUSDT" => "Avalanche",
            "LTCUSDT" => "Litecoin",
            "LINKUSDT" => "Chainlink",
            "XRPUSDT" => "XRP",
            "MATICUSDT" => "Polygon",
            "UNIUSDT" => "Uniswap",
            "ALGOUSDT" => "Algorand",
            "VETUSDT" => "VeChain",
            _ => "Unknown",
        };

        let symbol_text = Text::from(vec![
            Line::from(vec![
                Span::styled(&selected_crypto.symbol, Style::default().bold().fg(Color::Cyan)),
                Span::raw(" - "),
                Span::styled(symbol_name, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
        ]);
        let symbol_widget = Paragraph::new(symbol_text);
        f.render_widget(symbol_widget, details_layout[0]);

        // Current price (large and prominent)
        let price_text = Text::from(vec![
            Line::from(vec![
                Span::styled(
                    format!("${:.2}", selected_crypto.price),
                    Style::default().bold().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("USDT", Style::default().fg(Color::Gray)),
            ]),
        ]);
        let price_widget = Paragraph::new(price_text);
        f.render_widget(price_widget, details_layout[1]);

        // Price chart (sparkline)
        if !app.selected_candles.is_empty() {
            let chart_data: Vec<u64> = app.selected_candles.iter()
                .map(|candle| (candle.close * 100.0) as u64) // Convert to cents for better precision
                .collect();

            let sparkline = Sparkline::default()
                .block(Block::default().title("📈 5m Chart").borders(Borders::ALL))
                .data(&chart_data)
                .style(Style::default().fg(Color::Cyan));

            f.render_widget(sparkline, details_layout[2]);
        } else {
            let no_chart_text = Text::from(vec![
                Line::from("Loading chart data..."),
            ]);
            let no_chart_widget = Paragraph::new(no_chart_text)
                .block(Block::default().title("📈 5m Chart").borders(Borders::ALL));
            f.render_widget(no_chart_widget, details_layout[2]);
        }

        // 24h change
        let change_color = if selected_crypto.price_change_percent > 0.0 {
            Color::Green
        } else if selected_crypto.price_change_percent < 0.0 {
            Color::Red
        } else {
            Color::Gray
        };

        let change_symbol = if selected_crypto.price_change_percent > 0.0 {
            "📈"
        } else if selected_crypto.price_change_percent < 0.0 {
            "📉"
        } else {
            "➡️"
        };

        let change_text = Text::from(vec![
            Line::from(vec![
                Span::raw("24h Change: "),
                Span::styled(
                    format!("{} {:.2}%", change_symbol, selected_crypto.price_change_percent),
                    Style::default().fg(change_color).bold(),
                ),
            ]),
        ]);
        let change_widget = Paragraph::new(change_text);
        f.render_widget(change_widget, details_layout[3]);

        // 24h high/low
        let range_text = Text::from(vec![
            Line::from(vec![
                Span::raw("24h Range: "),
                Span::styled(
                    format!("H: ${:.2}", selected_crypto.high_24h),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(" / "),
                Span::styled(
                    format!("L: ${:.2}", selected_crypto.low_24h),
                    Style::default().fg(Color::Red),
                ),
            ]),
        ]);
        let range_widget = Paragraph::new(range_text);
        f.render_widget(range_widget, details_layout[4]);

        // Volume
        let volume_text = Text::from(vec![
            Line::from(vec![
                Span::raw("24h Volume: "),
                Span::styled(
                    format!("{:.0}", selected_crypto.volume),
                    Style::default().fg(Color::Blue).bold(),
                ),
            ]),
        ]);
        let volume_widget = Paragraph::new(volume_text);
        f.render_widget(volume_widget, details_layout[5]);
    } else {
        // No crypto selected (shouldn't happen, but just in case)
        let no_selection_text = Text::from(vec![
            Line::from("No cryptocurrency selected"),
        ]);
        let no_selection_widget = Paragraph::new(no_selection_text);
        f.render_widget(no_selection_widget, details_area);
    }
}

fn render_help_screen(f: &mut Frame, area: Rect) {
    // Render background overlay first (makes it opaque)
    let background = Block::default()
        .style(Style::default().bg(Color::Black));
    f.render_widget(background, area);

    // Create a centered help popup
    let popup_width = 60;
    let popup_height = 20;

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x,
        y,
        width: popup_width.min(area.width),
        height: popup_height.min(area.height),
    };

    let help_block = Block::default()
        .title("🎯 CoinPeek Help")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));

    f.render_widget(help_block.clone(), popup_area);

    let help_area = help_block.inner(popup_area);

    let help_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2), // Navigation
            Constraint::Length(2), // Sorting
            Constraint::Length(2), // Filtering
            Constraint::Length(2), // Data & Offline
            Constraint::Length(2), // General
            Constraint::Length(2), // Footer
        ])
        .split(help_area);

    // Navigation section
    let nav_text = Text::from(vec![
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" ↑/↓ Select | "),
            Span::styled("Mouse", Style::default().fg(Color::Magenta)),
            Span::raw(" Click to select"),
        ]),
        Line::from(vec![
            Span::raw("Search: "),
            Span::styled("/", Style::default().fg(Color::Green)),
            Span::raw(" Search mode | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" Exit search | "),
            Span::styled("?", Style::default().fg(Color::Cyan)),
            Span::raw(" Help"),
        ]),
    ]);
    let nav_widget = Paragraph::new(nav_text);
    f.render_widget(nav_widget, help_layout[0]);

    // Sorting section
    let sort_text = Text::from(vec![
        Line::from(vec![
            Span::styled("Sorting:", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" "),
            Span::styled("s", Style::default().fg(Color::Green)),
            Span::raw(" Cycle mode | "),
            Span::styled("d", Style::default().fg(Color::Green)),
            Span::raw(" Toggle direction"),
        ]),
    ]);
    let sort_widget = Paragraph::new(sort_text);
    f.render_widget(sort_widget, help_layout[1]);

    // Filtering section
    let filter_text = Text::from(vec![
        Line::from(vec![
            Span::styled("Filtering:", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" "),
            Span::styled("f", Style::default().fg(Color::Green)),
            Span::raw(" Cycle presets | "),
            Span::styled("c", Style::default().fg(Color::Green)),
            Span::raw(" Clear filters"),
        ]),
    ]);
    let filter_widget = Paragraph::new(filter_text);
    f.render_widget(filter_widget, help_layout[2]);

    // Data & Offline section
    let data_text = Text::from(vec![
        Line::from(vec![
            Span::styled("Data:", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" "),
            Span::styled("r", Style::default().fg(Color::Blue)),
            Span::raw(" Refresh | "),
            Span::styled("o", Style::default().fg(Color::Blue)),
            Span::raw(" Toggle offline | "),
            Span::styled("p", Style::default().fg(Color::Blue)),
            Span::raw(" Pause/resume"),
        ]),
    ]);
    let data_widget = Paragraph::new(data_text);
    f.render_widget(data_widget, help_layout[3]);

    // General section
    let general_text = Text::from(vec![
        Line::from(vec![
            Span::styled("General:", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" "),
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::raw(" Quit | "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red)),
            Span::raw(" Force quit"),
        ]),
    ]);
    let general_widget = Paragraph::new(general_text);
    f.render_widget(general_widget, help_layout[4]);

    // Footer
    let footer_text = Text::from(vec![
        Line::from(vec![
            Span::styled("💡 Tip:", Style::default().fg(Color::Gray)),
            Span::raw(" Press any key to close help"),
        ]),
    ]);
    let footer_widget = Paragraph::new(footer_text);
    f.render_widget(footer_widget, help_layout[5]);
}
