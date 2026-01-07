// src/ui.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Sparkline, Wrap},
    Frame,
};

use crate::app::App;

/// Draws the main crypto dashboard UI
pub fn render_dashboard(
    f: &mut Frame,
    area: Rect,
    app: &App,
) {
    // If alert management is active, show only the alert screen
    if app.show_alert_management {
        render_alert_management(f, area, app);
        return;
    }

    // If help is active, show only the help screen (clear the dashboard)
    if app.show_help {
        render_help_screen(f, area);
        return;
    }

    // Enhanced animated title with status
    let sync_status = app.get_offline_indicator();
    let (visible, total) = app.get_visible_count();
    let error_summary = app.get_error_summary();

    let mut title_parts = vec![
        Span::styled("🚀 ", Style::default().fg(Color::Yellow)),
        Span::styled("CoinPeek", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{}/{} coins", visible, total), Style::default().fg(Color::White)),
        Span::raw(" | "),
        Span::styled(&sync_status, Style::default().fg(match sync_status.chars().next() {
            Some('🟢') => Color::Green,
            Some('🟡') => Color::Yellow,
            Some('🔴') => Color::Red,
            _ => Color::Gray,
        })),
    ];

    if let Some(error) = &error_summary {
        title_parts.push(Span::raw(" | "));
        title_parts.push(Span::styled(error, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
    }

    let title_line = Line::from(title_parts);

    // Split area vertically: main content and footer
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Main content (minimum 10 lines)
            Constraint::Length(1),  // Footer hint
        ])
        .split(area);

    let main_block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));

    f.render_widget(main_block.clone(), vertical_layout[0]);

    let main_area = main_block.inner(vertical_layout[0]);

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

    // Footer hint
    let footer_text = Text::from(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled("?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" for help", Style::default().fg(Color::Gray)),
    ]));
    let footer_widget = Paragraph::new(footer_text)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(footer_widget, vertical_layout[1]);
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

        // Enhanced selection styling with borders and gradients
        if is_selected {
            // Render selection border/background first
            let selection_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Rgb(20, 20, 40))); // Dark blue gradient base

            f.render_widget(selection_block.clone(), layout[i]);

            // Selection indicator
            let indicator_block = Block::default()
                .borders(Borders::LEFT)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Rgb(40, 40, 80))); // Lighter blue

            let indicator_area = Rect {
                x: layout[i].x,
                y: layout[i].y,
                width: 2,
                height: layout[i].height,
            };
            f.render_widget(indicator_block, indicator_area);
        }

        // Create content with enhanced styling for selected items
        let mut price_line = vec![
            Span::raw(if is_selected { "▶ " } else { "  " }), // Selection arrow
            Span::styled(
                format!("{:<8}", price_info.symbol),
                Style::default()
                    .fg(if is_selected { Color::White } else { Color::Cyan })
                    .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() })
            ),
            Span::raw(": "),
            Span::styled(
                format!("${:.2}", price_info.price),
                Style::default()
                    .fg(if is_selected { Color::Yellow } else { Color::White })
                    .add_modifier(Modifier::BOLD)
            ),
            Span::raw("  "),
            Span::styled(
                format!("{} {:.2}%", change_symbol, price_info.price_change_percent),
                Style::default().fg(change_color).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
            ),
        ];

        let mut volume_line = vec![
            Span::raw(if is_selected { "  " } else { "           " }),
            Span::styled(
                format!("Vol: {:.0}", price_info.volume),
                Style::default()
                    .fg(if is_selected { Color::Blue } else { Color::Blue })
                    .add_modifier(if is_selected { Modifier::ITALIC } else { Modifier::empty() })
            ),
            Span::raw("  "),
            Span::styled(
                format!("H:{:.2} L:{:.2}", price_info.high_24h, price_info.low_24h),
                Style::default()
                    .fg(if is_selected { Color::Gray } else { Color::Gray })
                    .add_modifier(if is_selected { Modifier::DIM } else { Modifier::empty() })
            ),
        ];

        let price_line = Line::from(price_line);
        let volume_line = Line::from(volume_line);

        let text = Text::from(vec![price_line, volume_line]);
        let mut widget = Paragraph::new(text);

        // Add padding for selected items to account for borders
        if is_selected {
            widget = widget.block(Block::default().padding(ratatui::widgets::Padding::new(1, 1, 0, 0)));
        }

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

        // Price chart (sparkline) with loading animation
        let chart_area = details_layout[2];
        if !app.selected_candles.is_empty() {
            let chart_data: Vec<u64> = app.selected_candles.iter()
                .map(|candle| (candle.close * 100.0) as u64) // Convert to cents for better precision
                .collect();

            let sparkline = Sparkline::default()
                .block(Block::default()
                    .title("📈 5m Chart")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Green)))
                .data(&chart_data)
                .style(Style::default().fg(Color::Cyan));

            f.render_widget(sparkline, chart_area);
        } else {
            // Animated loading indicator
            let loading_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let frame_index = ((std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() / 100) % loading_frames.len() as u128) as usize;

            let loading_text = Text::from(vec![
                Line::from(vec![
                    Span::styled(loading_frames[frame_index], Style::default().fg(Color::Yellow)),
                    Span::raw(" "),
                    Span::styled("Loading chart data...", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Fetching latest price action", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                ]),
            ]);

            let loading_widget = Paragraph::new(loading_text)
                .block(Block::default()
                    .title("📈 5m Chart")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow)))
                .alignment(ratatui::layout::Alignment::Center);

            f.render_widget(loading_widget, chart_area);
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
            Span::styled("Ctrl+A", Style::default().fg(Color::Green)),
            Span::raw(" Alert management"),
        ]),
        Line::from(vec![
            Span::styled("?", Style::default().fg(Color::Cyan)),
            Span::raw(" Help | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" Exit modes"),
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

fn render_alert_management(f: &mut Frame, area: Rect, app: &App) {
    // Render background overlay first (makes it opaque)
    let background = Block::default()
        .style(Style::default().bg(Color::Black));
    f.render_widget(background, area);

    // Create a centered alert management popup
    let popup_width = 80;
    let popup_height = 25;

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x,
        y,
        width: popup_width.min(area.width),
        height: popup_height.min(area.height),
    };

    let alert_block = Block::default()
        .title("🔔 Price Alerts Management")
        .title_style(Style::default().fg(Color::Yellow).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));

    f.render_widget(alert_block.clone(), popup_area);

    let alert_area = alert_block.inner(popup_area);

    let alert_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2),  // Header with stats
            Constraint::Length(15), // Alert list
            Constraint::Length(3),  // Instructions
        ])
        .split(alert_area);

    // Header with alert statistics
    let enabled_count = app.get_enabled_alert_count();
    let total_count = app.alerts.len();
    let recent_count = app.get_recent_alerts().len();

    let header_text = Text::from(vec![
        Line::from(vec![
            Span::styled(format!("Active Alerts: {} | Total: {} | Recent: {}",
                               enabled_count, total_count, recent_count),
                         Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::styled("Recent Alerts:", Style::default().fg(Color::Yellow)),
        ]),
    ]);
    let header_widget = Paragraph::new(header_text);
    f.render_widget(header_widget, alert_layout[0]);

    // Alert list and recent notifications
    if app.alerts.is_empty() && app.get_recent_alerts().is_empty() {
        let empty_text = Text::from(vec![
            Line::from("No alerts configured"),
            Line::from(""),
            Line::from("Create alerts to get notified when prices hit your targets!"),
        ]);
        let empty_widget = Paragraph::new(empty_text)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(empty_widget, alert_layout[1]);
    } else {
        let mut alert_lines = Vec::new();

        // Show recent alerts first
        for (alert_msg, timestamp) in app.get_recent_alerts().iter().rev().take(3) {
            alert_lines.push(Line::from(vec![
                Span::styled("🔔 ", Style::default().fg(Color::Green)),
                Span::styled(alert_msg, Style::default().fg(Color::White)),
            ]));
        }

        if !app.get_recent_alerts().is_empty() && !app.alerts.is_empty() {
            alert_lines.push(Line::from(""));
        }

        // Show configured alerts
        for alert in &app.alerts {
            let status_icon = if alert.enabled { "🟢" } else { "🔴" };
            let condition_text = match &alert.condition {
                crate::app::AlertCondition::PriceAbove(threshold) =>
                    format!("Price > ${:.2}", threshold),
                crate::app::AlertCondition::PriceBelow(threshold) =>
                    format!("Price < ${:.2}", threshold),
                crate::app::AlertCondition::PercentChangeAbove(threshold) =>
                    format!("Change > {:.1}%", threshold),
                crate::app::AlertCondition::PercentChangeBelow(threshold) =>
                    format!("Change < {:.1}%", threshold),
                crate::app::AlertCondition::VolumeSpike(threshold) =>
                    format!("Volume > {:.0}", threshold),
            };

            alert_lines.push(Line::from(vec![
                Span::raw(status_icon),
                Span::raw(" "),
                Span::styled(&alert.symbol, Style::default().fg(Color::Cyan).bold()),
                Span::raw(" - "),
                Span::styled(condition_text, Style::default().fg(Color::Yellow)),
                Span::raw(format!(" ({} triggers)", alert.trigger_count)),
            ]));
        }

        let alert_text = Text::from(alert_lines);
        let alert_widget = Paragraph::new(alert_text);
        f.render_widget(alert_widget, alert_layout[1]);
    }

    // Instructions
    let instructions_text = Text::from(vec![
        Line::from(vec![
            Span::styled("Instructions:", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("• Create alerts to monitor price movements", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("• Alerts trigger terminal notifications", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("• Esc to close | This is a preview - full management coming!", Style::default().fg(Color::Gray)),
        ]),
    ]);
    let instructions_widget = Paragraph::new(instructions_text);
    f.render_widget(instructions_widget, alert_layout[2]);
}
