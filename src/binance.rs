use serde::Deserialize;
use reqwest::Error;

#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub price: String,
}

pub async fn fetch_price(symbol: &str) -> Result<f64, Error> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}",
        symbol
    );
    let resp = reqwest::get(&url).await?.json::<PriceResponse>().await?;
    let price = resp.price.parse::<f64>().unwrap_or(0.0);
    Ok(price)
}
