use serde::Deserialize;
use reqwest::Error;

#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    #[allow(dead_code)]
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Clone)]
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


/// Fetches 24hr ticker statistics for a single symbol
pub async fn fetch_24hr_stats(symbol: &str) -> Result<Ticker24hrResponse, Error> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/24hr?symbol={}",
        symbol
    );
    let resp = reqwest::get(&url).await?.json::<Ticker24hrResponse>().await?;
    Ok(resp)
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

/// Fetches comprehensive price information including 24hr stats for multiple symbols
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
