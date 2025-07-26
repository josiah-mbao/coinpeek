use serde::Deserialize;
use reqwest::Error;

#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    #[allow(dead_code)]
    pub symbol: String,
    pub price: String,
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
pub async fn fetch_prices(symbols: &[&str]) -> Vec<(String, f64)> {
    let fetches = symbols.iter().cloned().map(|symbol| async move {
        match fetch_price(&symbol).await {
            Ok(price) => (symbol.to_string(), price),
            Err(e) => {
                eprintln!("Failed to fetch {}: {}", symbol, e);
                (symbol.to_string(), 0.0)
            }
        }
    });

    futures::future::join_all(fetches).await
}
