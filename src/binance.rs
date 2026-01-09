use serde::Deserialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    #[allow(dead_code)]
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct Ticker24hrResponse {
    pub symbol: String,
    pub priceChange: String,
    pub priceChangePercent: String,
    pub weightedAvgPrice: String,
    pub prevClosePrice: String,
    pub lastPrice: String,
    pub lastQty: String,
    pub bidPrice: String,
    pub askPrice: String,
    pub openPrice: String,
    pub highPrice: String,
    pub lowPrice: String,
    pub volume: String,
    pub quoteVolume: String,
    pub openTime: u64,
    pub closeTime: u64,
    pub firstId: u64,
    pub lastId: u64,
    pub count: u64,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketPriceUpdate {
    pub stream: String,
    pub data: PriceData,
}

// For individual ticker streams, the message is directly the PriceData
pub type IndividualTickerUpdate = PriceData;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct PriceData {
    pub e: String, // Event type
    pub E: u64,    // Event time
    pub s: String, // Symbol
    pub p: String, // Price change
    pub P: String, // Price change percent
    pub w: String, // Weighted average price
    pub x: String, // Previous day's close price
    pub c: String, // Current price
    pub Q: String, // Current quantity
    pub b: String, // Best bid price
    pub B: String, // Best bid quantity
    pub a: String, // Best ask price
    pub A: String, // Best ask quantity
    pub o: String, // Open price
    pub h: String, // High price
    pub l: String, // Low price
    pub v: String, // Total traded base asset volume
    pub q: String, // Total traded quote asset volume
    pub O: u64,    // Statistics open time
    pub C: u64,    // Statistics close time
    pub F: u64,    // First trade ID
    pub L: u64,    // Last trade ID
    pub n: u64,    // Total number of trades
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PriceInfo {
    pub symbol: String,
    pub price: f64,
    pub price_change_percent: f64,
    pub volume: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub prev_close_price: f64,
}

#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: u64,
}

// Conditional compilation for HTTP clients
#[cfg(target_arch = "wasm32")]
use gloo::net::http::Request;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Error as ReqwestError;

#[cfg(target_arch = "wasm32")]
type Error = gloo::net::Error;
#[cfg(not(target_arch = "wasm32"))]
type Error = ReqwestError;

/// Validate that a symbol is safe for API calls
pub fn validate_symbol_for_api(symbol: &str) -> Result<(), String> {
    // Only allow uppercase letters and specific lengths
    if symbol.len() < 6 || symbol.len() > 14 {
        return Err("Symbol length must be 6-14 characters".to_string());
    }

    if !symbol.chars().all(|c| c.is_ascii_uppercase()) {
        return Err("Symbol must contain only uppercase ASCII letters".to_string());
    }

    // Prevent obvious injection attempts
    if symbol.contains("HTTP") || symbol.contains("://") || symbol.contains("<") || symbol.contains(">") {
        return Err("Symbol contains invalid characters".to_string());
    }

    Ok(())
}

/// Fetches the price of a single crypto symbol from Binance API
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_price(symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // Validate symbol before making API call
    validate_symbol_for_api(symbol)?;

    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}",
        symbol
    );
    let resp = reqwest::get(&url).await?.json::<PriceResponse>().await?;
    let price = resp.price.parse::<f64>().unwrap_or(0.0);
    Ok(price)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_price(symbol: &str) -> Result<f64, Error> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}",
        symbol
    );
    let resp = Request::get(&url).send().await?.json::<PriceResponse>().await?;
    let price = resp.price.parse::<f64>().unwrap_or(0.0);
    Ok(price)
}


/// Fetches 24hr ticker statistics for a single symbol
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_24hr_stats(symbol: &str) -> Result<Ticker24hrResponse, Error> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/24hr?symbol={}",
        symbol
    );
    let resp = reqwest::get(&url).await?.json::<Ticker24hrResponse>().await?;
    Ok(resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_24hr_stats(symbol: &str) -> Result<Ticker24hrResponse, Error> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/24hr?symbol={}",
        symbol
    );
    let resp = Request::get(&url).send().await?.json::<Ticker24hrResponse>().await?;
    Ok(resp)
}

/// Fetches prices for multiple symbols concurrently
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_prices(symbols: &[&str]) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let fetches = symbols.iter().cloned().map(|symbol| async move {
        match fetch_price(&symbol).await {
            Ok(price) => (symbol.to_string(), price),
            Err(e) => {
                eprintln!("Failed to fetch {}: {}", symbol, e);
                (symbol.to_string(), 0.0)
            }
        }
    });

    Ok(futures::future::join_all(fetches).await)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_prices(symbols: &[&str]) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let fetches = symbols.iter().cloned().map(|symbol| async move {
        match fetch_price(&symbol).await {
            Ok(price) => (symbol.to_string(), price),
            Err(e) => {
                web_sys::console::log_1(&format!("Failed to fetch {}: {:?}", symbol, e).into());
                (symbol.to_string(), 0.0)
            }
        }
    });

    Ok(futures::future::join_all(fetches).await)
}

/// Fetches comprehensive price information including 24hr stats for multiple symbols
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_price_infos(symbols: &[&str]) -> Result<Vec<PriceInfo>, Box<dyn std::error::Error>> {
    let fetches = symbols.iter().cloned().map(|symbol| async move {
        let (price_result, stats_result) = tokio::join!(
            fetch_price(&symbol),
            fetch_24hr_stats(&symbol)
        );

        match (price_result, stats_result) {
            (Ok(price), Ok(stats)) => PriceInfo {
                symbol: symbol.to_string(),
                price,
                price_change_percent: stats.priceChangePercent.parse().unwrap_or(0.0),
                volume: stats.volume.parse().unwrap_or(0.0),
                high_24h: stats.highPrice.parse().unwrap_or(0.0),
                low_24h: stats.lowPrice.parse().unwrap_or(0.0),
                prev_close_price: stats.prevClosePrice.parse().unwrap_or(0.0),
            },
            (Ok(price), Err(_)) => {
                eprintln!("Failed to fetch 24hr stats for {}, using basic price", symbol);
                PriceInfo {
                    symbol: symbol.to_string(),
                    price,
                    price_change_percent: 0.0,
                    volume: 0.0,
                    high_24h: 0.0,
                    low_24h: 0.0,
                    prev_close_price: 0.0,
                }
            },
            (Err(e), _) => {
                eprintln!("Failed to fetch {}: {}", symbol, e);
                PriceInfo {
                    symbol: symbol.to_string(),
                    price: 0.0,
                    price_change_percent: 0.0,
                    volume: 0.0,
                    high_24h: 0.0,
                    low_24h: 0.0,
                    prev_close_price: 0.0,
                }
            }
        }
    });

    Ok(futures::future::join_all(fetches).await)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_price_infos(symbols: &[&str]) -> Result<Vec<PriceInfo>, Box<dyn std::error::Error>> {
    let fetches = symbols.iter().cloned().map(|symbol| async move {
        let (price_result, stats_result) = futures::join!(
            fetch_price(&symbol),
            fetch_24hr_stats(&symbol)
        );

        match (price_result, stats_result) {
            (Ok(price), Ok(stats)) => PriceInfo {
                symbol: symbol.to_string(),
                price,
                price_change_percent: stats.priceChangePercent.parse().unwrap_or(0.0),
                volume: stats.volume.parse().unwrap_or(0.0),
                high_24h: stats.highPrice.parse().unwrap_or(0.0),
                low_24h: stats.lowPrice.parse().unwrap_or(0.0),
                prev_close_price: stats.prevClosePrice.parse().unwrap_or(0.0),
            },
            (Ok(price), Err(_)) => {
                web_sys::console::log_1(&format!("Failed to fetch 24hr stats for {}, using basic price", symbol).into());
                PriceInfo {
                    symbol: symbol.to_string(),
                    price,
                    price_change_percent: 0.0,
                    volume: 0.0,
                    high_24h: 0.0,
                    low_24h: 0.0,
                    prev_close_price: 0.0,
                }
            },
            (Err(e), _) => {
                web_sys::console::log_1(&format!("Failed to fetch {}: {:?}", symbol, e).into());
                PriceInfo {
                    symbol: symbol.to_string(),
                    price: 0.0,
                    price_change_percent: 0.0,
                    volume: 0.0,
                    high_24h: 0.0,
                    low_24h: 0.0,
                    prev_close_price: 0.0,
                }
            }
        }
    });

    Ok(futures::future::join_all(fetches).await)
}

/// Fetch candlestick (OHLC) data for a symbol over a given interval and number of points
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_candles(symbol: &str, interval: &str, limit: u8) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
        symbol, interval, limit
    );

    let raw_data = reqwest::get(&url).await?.json::<Vec<Vec<serde_json::Value>>>().await?;

    let candles = raw_data
        .into_iter()
        .filter_map(|entry| {
            Some(Candle {
                open: entry.get(1)?.as_str()?.parse().ok()?,
                high: entry.get(2)?.as_str()?.parse().ok()?,
                low: entry.get(3)?.as_str()?.parse().ok()?,
                close: entry.get(4)?.as_str()?.parse().ok()?,
                volume: entry.get(5)?.as_str()?.parse().ok()?,
                timestamp: entry.get(0)?.as_u64()?,
            })
        })
        .collect();

    Ok(candles)
}

/// Create WebSocket connection for real-time price updates
#[cfg(target_arch = "wasm32")]
pub fn create_price_websocket<F>(symbols: Vec<String>, on_message: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(IndividualTickerUpdate) + 'static,
{
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{ErrorEvent, MessageEvent, WebSocket};

    // Create streams parameter for Binance WebSocket
    let streams: Vec<String> = symbols.iter()
        .map(|symbol| format!("{}@ticker", symbol.to_lowercase()))
        .collect();

    let streams_param = streams.join("/");
    let url = format!("wss://stream.binance.com:9443/ws/{}", streams_param);

    web_sys::console::log_1(&format!("Connecting to WebSocket: {}", url).into());

    // Create WebSocket connection
    let ws = WebSocket::new(&url).map_err(|_| "Failed to create WebSocket")?;

    // Set binary type
    ws.set_binary_type(web_sys::BinaryType::Blob);

    // Clone on_message for the closure
    let on_message = std::rc::Rc::new(std::cell::RefCell::new(Some(on_message)));

    // Message handler
    {
        let on_message = on_message.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text_str = String::from(text);

                // Try parsing as IndividualTickerUpdate (direct PriceData) first
                if let Ok(update) = serde_json::from_str::<IndividualTickerUpdate>(&text_str) {
                    if let Some(ref callback) = *on_message.borrow() {
                        callback(update);
                    }
                } else {
                    web_sys::console::log_1(&format!("Failed to parse WebSocket message as IndividualTickerUpdate: {}", text_str).into());
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    // Error handler
    {
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            web_sys::console::log_1(&format!("WebSocket error: {:?}", e).into());
        }) as Box<dyn FnMut(ErrorEvent)>);

        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
    }

    // Open handler
    {
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            web_sys::console::log_1(&"WebSocket connection opened".into());
        }) as Box<dyn FnMut(web_sys::Event)>);

        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
    }

    // Close handler
    {
        let onclose_callback = Closure::wrap(Box::new(move |_| {
            web_sys::console::log_1(&"WebSocket connection closed".into());
        }) as Box<dyn FnMut(web_sys::Event)>);

        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
    }

    Ok(())
}

/// Convert WebSocket price data to PriceInfo
#[cfg(target_arch = "wasm32")]
pub fn websocket_data_to_price_info(data: &PriceData) -> PriceInfo {
    PriceInfo {
        symbol: data.s.clone(),
        price: data.c.parse().unwrap_or(0.0),
        price_change_percent: data.P.parse().unwrap_or(0.0),
        volume: data.v.parse().unwrap_or(0.0),
        high_24h: data.h.parse().unwrap_or(0.0),
        low_24h: data.l.parse().unwrap_or(0.0),
        prev_close_price: data.x.parse().unwrap_or(0.0),
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_candles(symbol: &str, interval: &str, limit: u8) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
        symbol, interval, limit
    );

    let raw_data = Request::get(&url).send().await?.json::<Vec<Vec<serde_json::Value>>>().await?;

    let candles = raw_data
        .into_iter()
        .filter_map(|entry| {
            Some(Candle {
                open: entry.get(1)?.as_str()?.parse().ok()?,
                high: entry.get(2)?.as_str()?.parse().ok()?,
                low: entry.get(3)?.as_str()?.parse().ok()?,
                close: entry.get(4)?.as_str()?.parse().ok()?,
                volume: entry.get(5)?.as_str()?.parse().ok()?,
                timestamp: entry.get(0)?.as_u64()?,
            })
        })
        .collect();

    Ok(candles)
}
