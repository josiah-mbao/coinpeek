use crate::binance::{PriceInfo, Candle};
use crate::config::Config;
use crate::database::Database;

#[derive(Debug, Clone, PartialEq)]
pub enum SortMode {
    Symbol,
    Price,
    ChangePercent,
    Volume,
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

pub struct App {
    pub price_infos: Vec<PriceInfo>,
    pub selected_index: usize,
    pub sort_mode: SortMode,
    pub paused: bool,
    pub config: Config,
    pub selected_candles: Vec<Candle>,
    pub selected_symbol_candles: String, // Track which symbol's candles we have
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            price_infos: Vec::new(),
            selected_index: 0,
            sort_mode: SortMode::Symbol,
            paused: false,
            config,
            selected_candles: Vec::new(),
            selected_symbol_candles: String::new(),
        }
    }

    pub fn update_prices(&mut self, mut price_infos: Vec<PriceInfo>) {
        // Sort the prices based on current sort mode
        self.sort_price_infos(&mut price_infos);

        self.price_infos = price_infos;

        // Ensure selected index is valid
        if self.selected_index >= self.price_infos.len() && !self.price_infos.is_empty() {
            self.selected_index = self.price_infos.len() - 1;
        }
    }

    fn sort_price_infos(&self, price_infos: &mut Vec<PriceInfo>) {
        match self.sort_mode {
            SortMode::Symbol => {
                price_infos.sort_by(|a, b| a.symbol.cmp(&b.symbol));
            }
            SortMode::Price => {
                price_infos.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
            }
            SortMode::ChangePercent => {
                price_infos.sort_by(|a, b| b.price_change_percent.partial_cmp(&a.price_change_percent).unwrap());
            }
            SortMode::Volume => {
                price_infos.sort_by(|a, b| b.volume.partial_cmp(&a.volume).unwrap());
            }
        }
    }

    pub fn next_sort_mode(&mut self) {
        self.sort_mode = self.sort_mode.next();
        // Re-sort with new mode
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
}
