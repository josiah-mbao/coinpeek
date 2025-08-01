use serde::Deserialize;
use reqwest::Error;

#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    #[allow(dead_code)]
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// Fetches the price of a single crypto symbol from Binance API
pub async fn fetch_price(symbol: &str) -> Result<f64, Error> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}",
        symbol
    );
    let resp = reqwest::get(&url).await?.json::<PriceResponse>().await?;
    let price = resp.price.parse::<f64>().unwrap_or(0.0);
    Ok(price)
}


/// Fetches prices for multiple symbols concurrently
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

    /// Fetch canldestick (OHLC) data for a symbol over a given interval and number of points
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
                open: entry[1].as_str()?.parse().ok()?,
                high: entry[2].as_str()?.parse().ok()?,
                low: entry[3].as_str()?.parse().ok()?,
                close: entry[4].as_str()?.parse().ok()?,

            })
        })
        .collect();

    Ok(candles)
}
