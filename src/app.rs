use crate::binance::{PriceInfo, Candle};
use crate::config::Config;
use crate::database::Database;
use chrono::{DateTime, Utc};

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
        }
    }

    pub fn update_prices(&mut self, price_infos: Vec<PriceInfo>) {
        // Store all price data
        self.all_price_infos = price_infos;

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
