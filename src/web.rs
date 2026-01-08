use crate::app::{App, SortDirection, AlertCondition};
use crate::binance::{PriceInfo, Candle};
use crate::config::Config;
use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::console;
use serde::{Deserialize, Serialize};
use gloo::timers::callback::Interval;

// WASM-JS interop for chart updates
#[wasm_bindgen]
extern "C" {
    fn updateCoinPeekChart(data: &str);
}

// Web-specific storage utilities
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoinPeekStorage {
    pub config: Config,
    pub price_data: Vec<PriceInfo>,
    pub last_update: Option<String>,
}

impl Default for CoinPeekStorage {
    fn default() -> Self {
        Self {
            config: Config::default(),
            price_data: Vec::new(),
            last_update: None,
        }
    }
}

pub struct WebApp {
    app: App,
    storage: CoinPeekStorage,
    _price_refresh_timer: Option<Interval>,
}

#[derive(Clone, Debug)]
pub enum TimeFrame {
    M1,   // 1 minute
    M5,   // 5 minutes
    M15,  // 15 minutes
    M30,  // 30 minutes
    H1,   // 1 hour
    H4,   // 4 hours
    D1,   // 1 day
    W1,   // 1 week
}

impl TimeFrame {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeFrame::M1 => "1m",
            TimeFrame::M5 => "5m",
            TimeFrame::M15 => "15m",
            TimeFrame::M30 => "30m",
            TimeFrame::H1 => "1h",
            TimeFrame::H4 => "4h",
            TimeFrame::D1 => "1d",
            TimeFrame::W1 => "1w",
        }
    }

    pub fn limit(&self) -> u8 {
        match self {
            TimeFrame::M1 => 100,
            TimeFrame::M5 => 100,
            TimeFrame::M15 => 100,
            TimeFrame::M30 => 100,
            TimeFrame::H1 => 100,
            TimeFrame::H4 => 100,
            TimeFrame::D1 => 100,
            TimeFrame::W1 => 100,
        }
    }
}

pub enum WebMsg {
    LoadFromStorage,
    SaveToStorage,
    UpdatePrices(Vec<PriceInfo>),
    SelectSymbol(usize),
    NextSortMode,
    ToggleSortDirection,
    NextFilter,
    ClearFilters,
    ToggleOffline,
    TogglePause,
    Search(String),
    RefreshData,
    CreateAlert(String, AlertCondition, Option<String>),
    LoadCandles(String, TimeFrame),
    UpdateCandles(Vec<Candle>),
    ChangeTimeFrame(TimeFrame),
    WebSocketUpdate(crate::binance::IndividualTickerUpdate),
    ConnectWebSocket,
    DisconnectWebSocket,
}

impl Component for WebApp {
    type Message = WebMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut app = App::new(Config::default());
        let storage = Self::load_from_local_storage().unwrap_or_default();

        // Load cached data
        if !storage.price_data.is_empty() {
            app.update_prices(storage.price_data.clone());
        }

        // Set up automatic price refresh timer (every 10 seconds)
        let link = ctx.link().clone();
        let price_refresh_timer = Some(Interval::new(10_000, move || {
            link.send_message(WebMsg::RefreshData);
        }));

        Self { app, storage, _price_refresh_timer: price_refresh_timer }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WebMsg::LoadFromStorage => {
                if let Ok(storage) = Self::load_from_local_storage() {
                    self.storage = storage;
                    if !self.storage.price_data.is_empty() {
                        self.app.update_prices(self.storage.price_data.clone());
                    }
                }
                true
            }
            WebMsg::SaveToStorage => {
                self.storage.price_data = self.app.all_price_infos.clone();
                self.storage.last_update = Some(chrono::Utc::now().to_rfc3339());
                let _ = Self::save_to_local_storage(&self.storage);
                true
            }
            WebMsg::UpdatePrices(prices) => {
                self.app.update_prices(prices.clone());
                self.storage.price_data = prices;
                self.storage.last_update = Some(chrono::Utc::now().to_rfc3339());
                let _ = Self::save_to_local_storage(&self.storage);
                true
            }
            WebMsg::SelectSymbol(index) => {
                self.app.selected_index = index;
                // Load candles for the selected symbol with default timeframe
                if let Some(selected) = self.app.get_selected_symbol() {
                    ctx.link().send_message(WebMsg::LoadCandles(selected.symbol.clone(), TimeFrame::M1));
                }
                true
            }
            WebMsg::NextSortMode => {
                self.app.next_sort_mode();
                true
            }
            WebMsg::ToggleSortDirection => {
                self.app.toggle_sort_direction();
                true
            }
            WebMsg::NextFilter => {
                self.app.next_filter_preset();
                true
            }
            WebMsg::ClearFilters => {
                self.app.clear_all_filters();
                true
            }
            WebMsg::ToggleOffline => {
                self.app.toggle_offline_mode();
                true
            }
            WebMsg::TogglePause => {
                self.app.toggle_pause();
                true
            }
            WebMsg::Search(query) => {
                if query.is_empty() {
                    self.app.exit_search_mode();
                } else {
                    // Clear existing search and set new query
                    self.app.active_filters.retain(|f| !matches!(f, crate::app::FilterType::SymbolSearch(_)));
                    self.app.active_filters.push(crate::app::FilterType::SymbolSearch(query));
                    self.app.apply_filters_and_sorting();
                }
                true
            }
            WebMsg::RefreshData => {
                // Trigger API refresh
                ctx.link().send_future(async {
                    match crate::binance::fetch_price_infos(&["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "SOLUSDT", "DOTUSDT", "DOGEUSDT", "AVAXUSDT", "LTCUSDT", "LINKUSDT"]).await {
                        Ok(prices) => WebMsg::UpdatePrices(prices),
                        Err(e) => {
                            console::log_1(&format!("API Error: {:?}", e).into());
                            WebMsg::LoadFromStorage
                        }
                    }
                });
                true
            }
            WebMsg::CreateAlert(symbol, condition, message) => {
                let _ = self.app.create_alert(symbol, condition, message);
                true
            }
            WebMsg::LoadCandles(symbol, timeframe) => {
                ctx.link().send_future(async move {
                    match crate::binance::fetch_candles(&symbol, timeframe.as_str(), timeframe.limit()).await {
                        Ok(candles) => WebMsg::UpdateCandles(candles),
                        Err(e) => {
                            web_sys::console::log_1(&format!("Failed to load candles: {:?}", e).into());
                            WebMsg::LoadFromStorage
                        }
                    }
                });
                true
            }
            WebMsg::UpdateCandles(candles) => {
                // Update chart with new candle data
                Self::update_chart(&candles);
                true
            }
            WebMsg::ChangeTimeFrame(timeframe) => {
                if let Some(selected) = self.app.get_selected_symbol() {
                    ctx.link().send_message(WebMsg::LoadCandles(selected.symbol.clone(), timeframe));
                }
                true
            }
            WebMsg::WebSocketUpdate(update) => {
                // Update price data from WebSocket
                let price_info = crate::binance::websocket_data_to_price_info(&update);
                self.app.update_prices(vec![price_info]);
                true
            }
            WebMsg::ConnectWebSocket => {
                // Connect to WebSocket for real-time updates
                let symbols = vec![
                    "BTCUSDT".to_string(),
                    "ETHUSDT".to_string(),
                    "BNBUSDT".to_string(),
                    "ADAUSDT".to_string(),
                    "SOLUSDT".to_string(),
                    "DOTUSDT".to_string(),
                    "DOGEUSDT".to_string(),
                    "AVAXUSDT".to_string(),
                    "LTCUSDT".to_string(),
                    "LINKUSDT".to_string(),
                ];

                let link = ctx.link().clone();
                ctx.link().send_future(async move {
                    let on_message = move |update: crate::binance::IndividualTickerUpdate| {
                        link.send_message(WebMsg::WebSocketUpdate(update));
                    };

                    if let Err(e) = crate::binance::create_price_websocket(symbols, on_message) {
                        console::log_1(&format!("WebSocket connection failed: {:?}", e).into());
                    }

                    WebMsg::LoadFromStorage // Dummy return
                });
                true
            }
            WebMsg::DisconnectWebSocket => {
                // WebSocket disconnection would be handled by the WebSocket library
                // For now, just log
                console::log_1(&"WebSocket disconnect requested".into());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        html! {
            <div class="coinpeek-container">
                <header class="coinpeek-header">
                    <h1>{ "🪙 CoinPeek" }</h1>
                    <div class="status-bar">
                        <span class="status">
                            { self.app.get_offline_indicator() }
                        </span>
                        <button onclick={link.callback(|_| WebMsg::RefreshData)}>
                            { "🔄 Refresh" }
                        </button>
                    </div>
                </header>

                <div class="controls">
                    <div class="control-group">
                        <label>{ "Sort: " }</label>
                        <button onclick={link.callback(|_| WebMsg::NextSortMode)}>
                            { format!("{} {}", self.app.sort_config.mode.as_str(),
                                match self.app.sort_config.direction {
                                    SortDirection::Ascending => "↑",
                                    SortDirection::Descending => "↓",
                                }) }
                        </button>
                        <button onclick={link.callback(|_| WebMsg::ToggleSortDirection)}>
                            { "↕️" }
                        </button>
                    </div>

                    <div class="control-group">
                        <label>{ "Filter: " }</label>
                        <button onclick={link.callback(|_| WebMsg::NextFilter)}>
                            { self.app.active_preset.as_str() }
                        </button>
                        <button onclick={link.callback(|_| WebMsg::ClearFilters)}>
                            { "🧹 Clear" }
                        </button>
                    </div>

                    <div class="control-group">
                        <label>{ "Mode: " }</label>
                        <button onclick={link.callback(|_| WebMsg::ToggleOffline)}>
                            { if self.app.data_status.offline_mode { "🔴 Offline" } else { "🟢 Online" } }
                        </button>
                        <button onclick={link.callback(|_| WebMsg::TogglePause)}>
                            { if self.app.paused { "⏸️ Paused" } else { "▶️ Running" } }
                        </button>
                        <button onclick={link.callback(|_| WebMsg::ConnectWebSocket)}>
                            { "🔗 WS Live" }
                        </button>
                    </div>

                    <div class="control-group">
                        <input
                            type="text"
                            placeholder="Search symbols..."
                            oninput={link.callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                WebMsg::Search(input.value())
                            })}
                        />
                    </div>
                </div>

                <div class="price-table">
                    <div class="table-header">
                        <div class="col-symbol">{ "Symbol" }</div>
                        <div class="col-price">{ "Price" }</div>
                        <div class="col-change">{ "24h Change" }</div>
                        <div class="col-volume">{ "Volume" }</div>
                    </div>

                    { for self.app.price_infos.iter().enumerate().map(|(index, price)| {
                        let is_selected = index == self.app.selected_index;
                        let onclick = link.callback(move |_| WebMsg::SelectSymbol(index));

                        html! {
                            <div class={classes!("table-row", if is_selected { "selected" } else { "" })} {onclick}>
                                <div class="col-symbol">{ &price.symbol }</div>
                                <div class="col-price">{ format!("${:.2}", price.price) }</div>
                                <div class={classes!("col-change", if price.price_change_percent >= 0.0 { "positive" } else { "negative" })}>
                                    { format!("{:+.2}%", price.price_change_percent) }
                                </div>
                                <div class="col-volume">{ format!("{:.0}", price.volume) }</div>
                            </div>
                        }
                    }) }
                </div>

                { if let Some(selected) = self.app.get_selected_symbol() {
                    html! {
                        <div class="selected-info">
                            <h3>{ format!("📊 {} Details", selected.symbol) }</h3>
                            <div class="details-grid">
                                <div>{ format!("Price: ${:.4}", selected.price) }</div>
                                <div>{ format!("High 24h: ${:.4}", selected.high_24h) }</div>
                                <div>{ format!("Low 24h: ${:.4}", selected.low_24h) }</div>
                                <div>{ format!("Volume: {:.0}", selected.volume) }</div>
                                <div>{ format!("Prev Close: ${:.4}", selected.prev_close_price) }</div>
                                <div>{ format!("Change: {:+.2}%", selected.price_change_percent) }</div>
                            </div>

                            <div class="chart-controls">
                                <div class="timeframe-buttons">
                                    <button class="timeframe-btn active" onclick={link.callback(|_| WebMsg::ChangeTimeFrame(TimeFrame::M1))}>{ "1m" }</button>
                                    <button class="timeframe-btn" onclick={link.callback(|_| WebMsg::ChangeTimeFrame(TimeFrame::M5))}>{ "5m" }</button>
                                    <button class="timeframe-btn" onclick={link.callback(|_| WebMsg::ChangeTimeFrame(TimeFrame::M15))}>{ "15m" }</button>
                                    <button class="timeframe-btn" onclick={link.callback(|_| WebMsg::ChangeTimeFrame(TimeFrame::H1))}>{ "1h" }</button>
                                    <button class="timeframe-btn" onclick={link.callback(|_| WebMsg::ChangeTimeFrame(TimeFrame::H4))}>{ "4h" }</button>
                                    <button class="timeframe-btn" onclick={link.callback(|_| WebMsg::ChangeTimeFrame(TimeFrame::D1))}>{ "1d" }</button>
                                </div>
                            </div>

                            <div class="price-chart">
                                <div id="chart-container" style="width: 100%; height: 400px;"></div>
                            </div>
                        </div>
                    }
                } else {
                    html! { <div></div> }
                } }

                { if !self.app.recent_alerts.is_empty() {
                    html! {
                        <div class="alerts">
                            <h3>{ "🔔 Recent Alerts" }</h3>
                            <ul>
                                { for self.app.recent_alerts.iter().map(|(message, timestamp)| {
                                    html! {
                                        <li>{ format!("{} - {}", timestamp.format("%H:%M:%S"), message) }</li>
                                    }
                                }) }
                            </ul>
                        </div>
                    }
                } else {
                    html! { <div></div> }
                } }
            </div>
        }
    }
}

impl WebApp {
    fn load_from_local_storage() -> Result<CoinPeekStorage, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let storage = window.local_storage()?.ok_or("No storage")?;

        let data = storage.get_item("coinpeek_data")?;
        match data {
            Some(json) => {
                serde_json::from_str(&json).map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))
            }
            None => Ok(CoinPeekStorage::default()),
        }
    }

    fn save_to_local_storage(data: &CoinPeekStorage) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let storage = window.local_storage()?.ok_or("No storage")?;

        let json = serde_json::to_string(data)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;

        storage.set_item("coinpeek_data", &json)
    }

    fn update_chart(candles: &[Candle]) {
        console::log_1(&format!("Rust update_chart called with {} candles", candles.len()).into());

        if candles.is_empty() {
            console::log_1(&"No candle data to update chart".into());
            return;
        }

        // First ensure chart is initialized
        Self::ensure_chart_initialized();

        // Convert candle data to format expected by Lightweight Charts
        #[derive(serde::Serialize)]
        struct ChartDataPoint {
            time: u64,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
        }

        let chart_data: Vec<ChartDataPoint> = candles
            .iter()
            .map(|candle| ChartDataPoint {
                time: candle.timestamp / 1000, // Convert ms to seconds for Lightweight Charts
                open: candle.open,
                high: candle.high,
                low: candle.low,
                close: candle.close,
            })
            .collect();

        console::log_1(&format!("Converted {} candles to chart format", chart_data.len()).into());

        // Serialize to JSON
        match serde_json::to_string(&chart_data) {
            Ok(json_data) => {
                console::log_1(&format!("Serialized to JSON, length: {}", json_data.len()).into());
                console::log_1(&format!("JSON preview: {}", &json_data[..json_data.len().min(200)]).into());
                updateCoinPeekChart(&json_data);
                console::log_1(&"Called updateCoinPeekChart".into());
            }
            Err(e) => {
                console::log_1(&format!("Failed to serialize chart data: {:?}", e).into());
            }
        }
    }

    fn ensure_chart_initialized() {
        console::log_1(&"Ensuring chart is initialized".into());

        // Call the JS initChart function if chart container exists
        // This is done via a small JS snippet injected through web_sys
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if document.get_element_by_id("chart-container").is_some() {
                    console::log_1(&"Chart container found, initializing chart".into());
                    // The chart should be initialized by the existing initChart function
                    // But since it runs on DOMContentLoaded, we need to call it manually
                    let _ = js_sys::eval("if (typeof initChart === 'function') { initChart(); console.log('Chart initialized from Rust'); } else { console.error('initChart function not found'); }");
                } else {
                    console::log_1(&"Chart container not found yet".into());
                }
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<WebApp>::new().render();
}
