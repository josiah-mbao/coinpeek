use crate::binance::{PriceInfo, Candle};
use crate::config::Config;
use crate::database::Database;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Network,      // Connection/internet issues
    Api,         // Binance API problems (rate limits, outages, invalid responses)
    Database,    // SQLite/storage issues
    Config,      // Configuration file problems
    Validation,  // Data validation/parsing errors
}

#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    Critical,    // App-breaking, requires immediate attention
    Warning,     // Degraded functionality, user should be aware
    Info,        // Minor issues, mostly informational
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub error_type: ErrorType,
    pub severity: ErrorSeverity,
    pub message: String,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
    pub retry_count: u32,
    pub recovery_suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortMode {
    Symbol,
    Price,
    ChangePercent,
    Volume,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortConfig {
    pub mode: SortMode,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    PriceRange { min: Option<f64>, max: Option<f64> },
    ChangePercentRange { min: Option<f64>, max: Option<f64> },
    VolumeRange { min: Option<f64>, max: Option<f64> },
    SymbolSearch(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertCondition {
    PriceAbove(f64),        // Alert when price > threshold
    PriceBelow(f64),        // Alert when price < threshold
    PercentChangeAbove(f64), // Alert when % change > threshold (positive)
    PercentChangeBelow(f64), // Alert when % change < threshold (negative)
    VolumeSpike(f64),       // Alert when volume > threshold
}

#[derive(Debug, Clone)]
pub struct PriceAlert {
    pub id: u32,
    pub symbol: String,
    pub condition: AlertCondition,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub trigger_count: u32,
    pub message: Option<String>, // Custom alert message
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterPreset {
    All,                    // No filters
    TopGainers,            // > 5% up in 24h
    TopLosers,             // < -5% down in 24h
    HighVolume,            // Top 20% by volume
    Volatile,              // High volatility (>3% change)
    Stable,                // Low volatility (<1% change)
}

impl FilterPreset {
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterPreset::All => "All Coins",
            FilterPreset::TopGainers => "Top Gainers",
            FilterPreset::TopLosers => "Top Losers",
            FilterPreset::HighVolume => "High Volume",
            FilterPreset::Volatile => "Volatile",
            FilterPreset::Stable => "Stable",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            FilterPreset::All => FilterPreset::TopGainers,
            FilterPreset::TopGainers => FilterPreset::TopLosers,
            FilterPreset::TopLosers => FilterPreset::HighVolume,
            FilterPreset::HighVolume => FilterPreset::Volatile,
            FilterPreset::Volatile => FilterPreset::Stable,
            FilterPreset::Stable => FilterPreset::All,
        }
    }
}

impl SortMode {
    pub fn next(&self) -> Self {
        match self {
            SortMode::Symbol => SortMode::Price,
            SortMode::Price => SortMode::ChangePercent,
            SortMode::ChangePercent => SortMode::Volume,
            SortMode::Volume => SortMode::Symbol,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SortMode::Symbol => "Symbol",
            SortMode::Price => "Price",
            SortMode::ChangePercent => "24h Change",
            SortMode::Volume => "Volume",
        }
    }
}

impl SortConfig {
    pub fn new(mode: SortMode, direction: SortDirection) -> Self {
        Self { mode, direction }
    }

    pub fn default() -> Self {
        Self {
            mode: SortMode::Symbol,
            direction: SortDirection::Ascending,
        }
    }

    pub fn next_mode(&mut self) {
        self.mode = self.mode.next();
    }

    pub fn toggle_direction(&mut self) {
        self.direction = match self.direction {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        };
    }

    pub fn display_name(&self) -> String {
        let direction_indicator = match self.direction {
            SortDirection::Ascending => "↑",
            SortDirection::Descending => "↓",
        };
        format!("{} {}", self.mode.as_str(), direction_indicator)
    }
}

#[derive(Debug, Clone)]
pub struct DataStatus {
    pub last_price_update: Option<DateTime<Utc>>,
    pub last_successful_sync: Option<DateTime<Utc>>,
    pub offline_mode: bool,
    pub consecutive_failures: u32,
}

pub struct App {
    pub all_price_infos: Vec<PriceInfo>,      // All available price data
    pub price_infos: Vec<PriceInfo>,          // Currently filtered and sorted data
    pub selected_index: usize,
    pub sort_config: SortConfig,
    pub active_filters: Vec<FilterType>,
    pub active_preset: FilterPreset,
    pub paused: bool,
    pub config: Config,
    pub selected_candles: Vec<Candle>,
    pub selected_symbol_candles: String, // Track which symbol's candles we have
    pub data_status: DataStatus,         // Track data freshness and offline status
    pub show_help: bool,                 // Show help overlay
    pub search_mode: bool,               // Interactive search mode
    pub search_query: String,            // Current search query
    pub show_alert_management: bool,     // Show alert management screen
    pub errors: Vec<AppError>,           // Active application errors
    pub alerts: Vec<PriceAlert>,         // Price alerts
    pub recent_alerts: Vec<(String, DateTime<Utc>)>, // Recently triggered alerts (message, timestamp)
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            all_price_infos: Vec::new(),
            price_infos: Vec::new(),
            selected_index: 0,
            sort_config: SortConfig::default(),
            active_filters: Vec::new(),
            active_preset: FilterPreset::All,
            paused: false,
            config,
            selected_candles: Vec::new(),
            selected_symbol_candles: String::new(),
            data_status: DataStatus {
                last_price_update: None,
                last_successful_sync: None,
                offline_mode: false,
                consecutive_failures: 0,
            },
            show_help: false,
            search_mode: false,
            search_query: String::new(),
            show_alert_management: false,
            errors: Vec::new(),
            alerts: Vec::new(),
            recent_alerts: Vec::new(),
        }
    }

    pub fn update_prices(&mut self, price_infos: Vec<PriceInfo>) {
        // Store all price data
        self.all_price_infos = price_infos;

        // Check alerts against new price data
        self.check_alerts();

        // Apply filters and sorting
        self.apply_filters_and_sorting();

        // Ensure selected index is valid
        if self.selected_index >= self.price_infos.len() && !self.price_infos.is_empty() {
            self.selected_index = self.price_infos.len() - 1;
        }
    }

    fn apply_filters_and_sorting(&mut self) {
        let mut filtered = self.all_price_infos.clone();

        // Apply preset filters first
        self.apply_preset_filters(&mut filtered);

        // Apply custom filters
        self.apply_custom_filters(&mut filtered);

        // Sort the filtered results
        self.sort_price_infos(&mut filtered);

        self.price_infos = filtered;
    }

    fn apply_preset_filters(&self, price_infos: &mut Vec<PriceInfo>) {
        match self.active_preset {
            FilterPreset::All => {} // No filtering
            FilterPreset::TopGainers => {
                price_infos.retain(|p| p.price_change_percent > 5.0);
            }
            FilterPreset::TopLosers => {
                price_infos.retain(|p| p.price_change_percent < -5.0);
            }
            FilterPreset::HighVolume => {
                if !price_infos.is_empty() {
                    // Keep top 20% by volume
                    let mut sorted_by_volume = price_infos.clone();
                    sorted_by_volume.sort_by(|a, b| b.volume.partial_cmp(&a.volume).unwrap());
                    let top_count = (sorted_by_volume.len() as f64 * 0.2).ceil() as usize;
                    let min_volume = sorted_by_volume.get(top_count.saturating_sub(1))
                        .map(|p| p.volume)
                        .unwrap_or(0.0);
                    price_infos.retain(|p| p.volume >= min_volume);
                }
            }
            FilterPreset::Volatile => {
                price_infos.retain(|p| p.price_change_percent.abs() > 3.0);
            }
            FilterPreset::Stable => {
                price_infos.retain(|p| p.price_change_percent.abs() < 1.0);
            }
        }
    }

    fn apply_custom_filters(&self, price_infos: &mut Vec<PriceInfo>) {
        for filter in &self.active_filters {
            match filter {
                FilterType::PriceRange { min, max } => {
                    price_infos.retain(|p| {
                        let price = p.price;
                        min.map_or(true, |min_val| price >= min_val) &&
                        max.map_or(true, |max_val| price <= max_val)
                    });
                }
                FilterType::ChangePercentRange { min, max } => {
                    price_infos.retain(|p| {
                        let change = p.price_change_percent;
                        min.map_or(true, |min_val| change >= min_val) &&
                        max.map_or(true, |max_val| change <= max_val)
                    });
                }
                FilterType::VolumeRange { min, max } => {
                    price_infos.retain(|p| {
                        let volume = p.volume;
                        min.map_or(true, |min_val| volume >= min_val) &&
                        max.map_or(true, |max_val| volume <= max_val)
                    });
                }
                FilterType::SymbolSearch(search_term) => {
                    if !search_term.is_empty() {
                        price_infos.retain(|p|
                            p.symbol.to_lowercase().contains(&search_term.to_lowercase())
                        );
                    }
                }
            }
        }
    }

    fn sort_price_infos(&self, price_infos: &mut Vec<PriceInfo>) {
        match (&self.sort_config.mode, &self.sort_config.direction) {
            (SortMode::Symbol, SortDirection::Ascending) => {
                price_infos.sort_by(|a, b| a.symbol.cmp(&b.symbol));
            }
            (SortMode::Symbol, SortDirection::Descending) => {
                price_infos.sort_by(|a, b| b.symbol.cmp(&a.symbol));
            }
            (SortMode::Price, SortDirection::Ascending) => {
                price_infos.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
            }
            (SortMode::Price, SortDirection::Descending) => {
                price_infos.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
            }
            (SortMode::ChangePercent, SortDirection::Ascending) => {
                price_infos.sort_by(|a, b| a.price_change_percent.partial_cmp(&b.price_change_percent).unwrap());
            }
            (SortMode::ChangePercent, SortDirection::Descending) => {
                price_infos.sort_by(|a, b| b.price_change_percent.partial_cmp(&a.price_change_percent).unwrap());
            }
            (SortMode::Volume, SortDirection::Ascending) => {
                price_infos.sort_by(|a, b| a.volume.partial_cmp(&b.volume).unwrap());
            }
            (SortMode::Volume, SortDirection::Descending) => {
                price_infos.sort_by(|a, b| b.volume.partial_cmp(&a.volume).unwrap());
            }
        }
    }

    pub fn next_sort_mode(&mut self) {
        self.sort_config.next_mode();
        // Re-sort with new mode
        if !self.price_infos.is_empty() {
            let mut sorted = self.price_infos.clone();
            self.sort_price_infos(&mut sorted);
            self.price_infos = sorted;
        }
    }

    pub fn toggle_sort_direction(&mut self) {
        self.sort_config.toggle_direction();
        // Re-sort with new direction
        if !self.price_infos.is_empty() {
            let mut sorted = self.price_infos.clone();
            self.sort_price_infos(&mut sorted);
            self.price_infos = sorted;
        }
    }

    pub fn select_next(&mut self) {
        if !self.price_infos.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.price_infos.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.price_infos.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.price_infos.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        // Clear any existing symbol search filters when entering search mode
        self.active_filters.retain(|f| !matches!(f, FilterType::SymbolSearch(_)));
    }

    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        // Clear search filter when exiting
        self.active_filters.retain(|f| !matches!(f, FilterType::SymbolSearch(_)));
        self.apply_filters_and_sorting();
    }

    pub fn update_search_query(&mut self, c: char) {
        self.search_query.push(c);
        self.apply_search_filter();
    }

    pub fn backspace_search(&mut self) {
        self.search_query.pop();
        self.apply_search_filter();
    }

    fn apply_search_filter(&mut self) {
        // Remove any existing symbol search filters
        self.active_filters.retain(|f| !matches!(f, FilterType::SymbolSearch(_)));

        if !self.search_query.is_empty() {
            // Add new search filter
            self.active_filters.push(FilterType::SymbolSearch(self.search_query.clone()));
        }

        // Re-apply all filters and sorting
        self.apply_filters_and_sorting();

        // Reset selection if it's now out of bounds
        if self.selected_index >= self.price_infos.len() && !self.price_infos.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn get_selected_symbol(&self) -> Option<&PriceInfo> {
        self.price_infos.get(self.selected_index)
    }

    pub fn update_candles_for_selected(&mut self, candles: Vec<Candle>) {
        if let Some(selected) = self.price_infos.get(self.selected_index) {
            self.selected_candles = candles;
            self.selected_symbol_candles = selected.symbol.clone();
        }
    }

    pub fn should_fetch_candles(&self) -> Option<String> {
        if let Some(selected) = self.get_selected_symbol() {
            if self.selected_symbol_candles != selected.symbol || self.selected_candles.is_empty() {
                return Some(selected.symbol.clone());
            }
        }
        None
    }

    // Filter and preset management methods
    pub fn next_filter_preset(&mut self) {
        self.active_preset = self.active_preset.next();
        self.apply_filters_and_sorting();

        // Reset selection if it's now out of bounds
        if self.selected_index >= self.price_infos.len() && !self.price_infos.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn set_filter_preset(&mut self, preset: FilterPreset) {
        self.active_preset = preset;
        self.apply_filters_and_sorting();

        // Reset selection if it's now out of bounds
        if self.selected_index >= self.price_infos.len() && !self.price_infos.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn add_filter(&mut self, filter: FilterType) {
        // Remove any existing filter of the same type
        self.active_filters.retain(|f| !matches_filter_type(f, &filter));
        self.active_filters.push(filter);
        self.apply_filters_and_sorting();

        // Reset selection if it's now out of bounds
        if self.selected_index >= self.price_infos.len() && !self.price_infos.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn remove_filter(&mut self, filter_type: &FilterType) {
        self.active_filters.retain(|f| !matches_filter_type(f, filter_type));
        self.apply_filters_and_sorting();

        // Reset selection if it's now out of bounds
        if self.selected_index >= self.price_infos.len() && !self.price_infos.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn clear_all_filters(&mut self) {
        self.active_filters.clear();
        self.active_preset = FilterPreset::All;
        self.apply_filters_and_sorting();

        // Reset selection
        self.selected_index = 0;
    }

    pub fn get_filter_status(&self) -> String {
        if self.active_filters.is_empty() && matches!(self.active_preset, FilterPreset::All) {
            "No filters active".to_string()
        } else {
            let preset_text = if matches!(self.active_preset, FilterPreset::All) {
                String::new()
            } else {
                format!("Preset: {}", self.active_preset.as_str())
            };

            let filter_count = self.active_filters.len();
            let filter_text = if filter_count > 0 {
                format!("{} custom filter{}", filter_count, if filter_count == 1 { "" } else { "s" })
            } else {
                String::new()
            };

            match (preset_text.is_empty(), filter_text.is_empty()) {
                (true, true) => "No filters active".to_string(),
                (false, true) => preset_text,
                (true, false) => filter_text,
                (false, false) => format!("{}, {}", preset_text, filter_text),
            }
        }
    }

    pub fn get_visible_count(&self) -> (usize, usize) {
        (self.price_infos.len(), self.all_price_infos.len())
    }

    // Data status and offline awareness methods
    pub fn record_successful_sync(&mut self) {
        let now = Utc::now();
        self.data_status.last_successful_sync = Some(now);
        self.data_status.last_price_update = Some(now);
        self.data_status.consecutive_failures = 0;
    }

    pub fn record_sync_failure(&mut self) {
        self.data_status.consecutive_failures += 1;
        if self.data_status.consecutive_failures >= 3 {
            self.data_status.offline_mode = true;
        }
    }

    pub fn toggle_offline_mode(&mut self) {
        self.data_status.offline_mode = !self.data_status.offline_mode;
        if self.data_status.offline_mode {
            self.data_status.consecutive_failures = 0; // Reset failure count when manually enabling offline mode
        }
    }

    pub fn get_data_age_string(&self) -> String {
        match self.data_status.last_successful_sync {
            Some(last_sync) => {
                let now = Utc::now();
                let duration = now.signed_duration_since(last_sync);

                if duration.num_minutes() < 1 {
                    "just now".to_string()
                } else if duration.num_minutes() < 60 {
                    format!("{}m ago", duration.num_minutes())
                } else if duration.num_hours() < 24 {
                    format!("{}h ago", duration.num_hours())
                } else {
                    format!("{}d ago", duration.num_days())
                }
            }
            None => "never".to_string(),
        }
    }

    pub fn get_offline_indicator(&self) -> String {
        if self.data_status.offline_mode {
            "🔴 OFFLINE".to_string()
        } else if self.data_status.consecutive_failures > 0 {
            format!("🟡 {} failures", self.data_status.consecutive_failures)
        } else {
            format!("🟢 synced {}", self.get_data_age_string())
        }
    }

    pub fn is_data_stale(&self) -> bool {
        match self.data_status.last_successful_sync {
            Some(last_sync) => {
                let now = Utc::now();
                let duration = now.signed_duration_since(last_sync);
                duration.num_minutes() > 30 // Consider data stale after 30 minutes
            }
            None => true,
        }
    }

    // Error management methods
    pub fn add_error(&mut self, error_type: ErrorType, severity: ErrorSeverity, message: String, details: Option<String>, recovery_suggestion: Option<String>) {
        let error = AppError {
            error_type,
            severity,
            message,
            details,
            timestamp: Utc::now(),
            resolved: false,
            retry_count: 0,
            recovery_suggestion,
        };
        self.errors.push(error);
    }

    pub fn resolve_error(&mut self, index: usize) {
        if let Some(error) = self.errors.get_mut(index) {
            error.resolved = true;
        }
    }

    pub fn clear_resolved_errors(&mut self) {
        self.errors.retain(|e| !e.resolved);
    }

    pub fn get_active_error_count(&self) -> usize {
        self.errors.iter().filter(|e| !e.resolved).count()
    }

    pub fn get_error_summary(&self) -> Option<String> {
        let active_errors: Vec<&AppError> = self.errors.iter().filter(|e| !e.resolved).collect();

        if active_errors.is_empty() {
            return None;
        }

        let critical_count = active_errors.iter().filter(|e| matches!(e.severity, ErrorSeverity::Critical)).count();
        let warning_count = active_errors.iter().filter(|e| matches!(e.severity, ErrorSeverity::Warning)).count();

        let mut summary_parts = Vec::new();

        if critical_count > 0 {
            summary_parts.push(format!("🚨 {} critical", critical_count));
        }
        if warning_count > 0 {
            summary_parts.push(format!("⚠️ {} warnings", warning_count));
        }

        if !summary_parts.is_empty() {
            Some(summary_parts.join(", "))
        } else {
            Some(format!("ℹ️ {} info", active_errors.len()))
        }
    }

    pub fn retry_failed_operations(&mut self) {
        // Mark all errors as having been retried
        for error in &mut self.errors {
            if !error.resolved {
                error.retry_count += 1;
            }
        }
        // Note: Actual retry logic would be implemented in the main loop
    }

    // Convenience methods for common error types
    pub fn add_network_error(&mut self, message: String, details: Option<String>) {
        self.add_error(
            ErrorType::Network,
            ErrorSeverity::Warning,
            message,
            details,
            Some("Check your internet connection".to_string()),
        );
    }

    pub fn add_api_error(&mut self, message: String, details: Option<String>) {
        self.add_error(
            ErrorType::Api,
            ErrorSeverity::Warning,
            message,
            details,
            Some("API may be temporarily unavailable - data will refresh when available".to_string()),
        );
    }

    pub fn add_database_error(&mut self, message: String, details: Option<String>) {
        self.add_error(
            ErrorType::Database,
            ErrorSeverity::Critical,
            message,
            details,
            Some("Application may run in limited mode - restart may help".to_string()),
        );
    }

    pub fn add_config_error(&mut self, message: String, details: Option<String>) {
        self.add_error(
            ErrorType::Config,
            ErrorSeverity::Critical,
            message,
            details,
            Some("Check coinpeek.json configuration file".to_string()),
        );
    }

    // Alert management methods
    pub fn create_alert(&mut self, symbol: String, condition: AlertCondition, message: Option<String>) -> u32 {
        let id = self.alerts.len() as u32 + 1;
        let alert = PriceAlert {
            id,
            symbol,
            condition,
            enabled: true,
            created_at: Utc::now(),
            last_triggered: None,
            trigger_count: 0,
            message,
        };
        self.alerts.push(alert);
        id
    }

    pub fn delete_alert(&mut self, id: u32) -> bool {
        let initial_len = self.alerts.len();
        self.alerts.retain(|alert| alert.id != id);
        self.alerts.len() < initial_len
    }

    pub fn toggle_alert(&mut self, id: u32) -> bool {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.id == id) {
            alert.enabled = !alert.enabled;
            true
        } else {
            false
        }
    }

    pub fn check_alerts(&mut self) {
        for alert in &mut self.alerts {
            if !alert.enabled {
                continue;
            }

            // Find the price info for this symbol
            if let Some(price_info) = self.all_price_infos.iter().find(|p| p.symbol == alert.symbol) {
                let should_trigger = match &alert.condition {
                    AlertCondition::PriceAbove(threshold) => price_info.price > *threshold,
                    AlertCondition::PriceBelow(threshold) => price_info.price < *threshold,
                    AlertCondition::PercentChangeAbove(threshold) => price_info.price_change_percent > *threshold,
                    AlertCondition::PercentChangeBelow(threshold) => price_info.price_change_percent < *threshold,
                    AlertCondition::VolumeSpike(threshold) => price_info.volume > *threshold,
                };

                if should_trigger {
                    // Check if we've already triggered this alert recently (avoid spam)
                    let should_notify = match alert.last_triggered {
                        Some(last_trigger) => {
                            let now = Utc::now();
                            let duration = now.signed_duration_since(last_trigger);
                            // Only trigger once per hour for the same alert
                            duration.num_hours() >= 1
                        }
                        None => true, // Never triggered before
                    };

                    if should_notify {
                        alert.last_triggered = Some(Utc::now());
                        alert.trigger_count += 1;

                        // Create notification message
                        let message = alert.message.clone().unwrap_or_else(|| {
                            match &alert.condition {
                                AlertCondition::PriceAbove(threshold) => {
                                    format!("{} price above ${:.2} (currently ${:.2})", alert.symbol, threshold, price_info.price)
                                }
                                AlertCondition::PriceBelow(threshold) => {
                                    format!("{} price below ${:.2} (currently ${:.2})", alert.symbol, threshold, price_info.price)
                                }
                                AlertCondition::PercentChangeAbove(threshold) => {
                                    format!("{} up {:.1}% (currently {:.2}%)", alert.symbol, threshold, price_info.price_change_percent)
                                }
                                AlertCondition::PercentChangeBelow(threshold) => {
                                    format!("{} down {:.1}% (currently {:.2}%)", alert.symbol, threshold, price_info.price_change_percent)
                                }
                                AlertCondition::VolumeSpike(threshold) => {
                                    format!("{} volume spike: {:.0} (threshold: {:.0})", alert.symbol, price_info.volume, threshold)
                                }
                            }
                        });

                        // Terminal bell notification
                        print!("\x07"); // ASCII bell character

                        // Add to recent alerts for notification
                        self.recent_alerts.push((format!("🔔 {}", message), Utc::now()));

                        // Keep only the last 10 recent alerts
                        if self.recent_alerts.len() > 10 {
                            self.recent_alerts.remove(0);
                        }
                    }
                }
            }
        }
    }

    pub fn get_enabled_alert_count(&self) -> usize {
        self.alerts.iter().filter(|a| a.enabled).count()
    }

    pub fn get_recent_alerts(&self) -> &[(String, DateTime<Utc>)] {
        &self.recent_alerts
    }

    pub fn clear_recent_alerts(&mut self) {
        self.recent_alerts.clear();
    }
}

// Helper function to check if two filters are of the same type
fn matches_filter_type(existing: &FilterType, new: &FilterType) -> bool {
    matches!(
        (existing, new),
        (FilterType::PriceRange { .. }, FilterType::PriceRange { .. }) |
        (FilterType::ChangePercentRange { .. }, FilterType::ChangePercentRange { .. }) |
        (FilterType::VolumeRange { .. }, FilterType::VolumeRange { .. }) |
        (FilterType::SymbolSearch(_), FilterType::SymbolSearch(_))
    )
}
