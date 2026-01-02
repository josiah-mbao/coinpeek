use rusqlite::{Connection, Result as SqlResult, params, OptionalExtension};
use tokio_rusqlite::Connection as AsyncConnection;
use chrono::{DateTime, Utc};
use std::path::Path;
use crate::binance::{PriceInfo, Candle};

/// Database connection manager
pub struct Database {
    conn: AsyncConnection,
}

impl Database {
    /// Create a new database connection and initialize schema
    pub async fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = AsyncConnection::open(db_path).await?;

        // Enable WAL mode for better concurrency
        conn.call(|conn| {
            // Execute PRAGMA statements that don't return results
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA cache_size = 1000000;
                 PRAGMA temp_store = MEMORY;"
            )?;
            Ok(())
        }).await?;

        // Initialize schema
        Self::init_schema(&conn).await?;

        Ok(Database { conn })
    }

    /// Initialize database schema
    async fn init_schema(conn: &AsyncConnection) -> Result<(), Box<dyn std::error::Error>> {
        conn.call(|conn| {
            // Prices table for current price data
            conn.execute(
                "CREATE TABLE IF NOT EXISTS prices (
                    id INTEGER PRIMARY KEY,
                    symbol TEXT NOT NULL,
                    price REAL NOT NULL,
                    price_change_percent REAL,
                    volume REAL,
                    high_24h REAL,
                    low_24h REAL,
                    prev_close_price REAL,
                    timestamp INTEGER NOT NULL,
                    exchange TEXT DEFAULT 'binance',
                    created_at INTEGER DEFAULT (strftime('%s', 'now'))
                )",
                [],
            )?;

            // Candles table for historical OHLC data
            conn.execute(
                "CREATE TABLE IF NOT EXISTS candles (
                    id INTEGER PRIMARY KEY,
                    symbol TEXT NOT NULL,
                    timeframe TEXT NOT NULL,
                    open REAL NOT NULL,
                    high REAL NOT NULL,
                    low REAL NOT NULL,
                    close REAL NOT NULL,
                    volume REAL,
                    timestamp INTEGER NOT NULL,
                    exchange TEXT DEFAULT 'binance',
                    created_at INTEGER DEFAULT (strftime('%s', 'now'))
                )",
                [],
            )?;

            // Sync metadata for tracking last sync times
            conn.execute(
                "CREATE TABLE IF NOT EXISTS sync_metadata (
                    key TEXT PRIMARY KEY,
                    value TEXT,
                    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
                )",
                [],
            )?;

            // Indexes for performance
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_prices_symbol_timestamp
                ON prices(symbol, timestamp)",
                [],
            )?;

            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_candles_symbol_timeframe_timestamp
                ON candles(symbol, timeframe, timestamp)",
                [],
            )?;

            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_prices_timestamp
                ON prices(timestamp)",
                [],
            )?;

            Ok(())
        }).await?;

        Ok(())
    }

    /// Store price information
    pub async fn store_price_info(&self, price_info: &PriceInfo) -> Result<(), Box<dyn std::error::Error>> {
        let symbol = price_info.symbol.clone();
        let price = price_info.price;
        let price_change_percent = price_info.price_change_percent;
        let volume = price_info.volume;
        let high_24h = price_info.high_24h;
        let low_24h = price_info.low_24h;
        let prev_close_price = price_info.prev_close_price;
        let timestamp = Utc::now().timestamp();

        self.conn.call(move |conn| {
            conn.execute(
                "INSERT INTO prices (
                    symbol, price, price_change_percent, volume,
                    high_24h, low_24h, prev_close_price, timestamp
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    symbol,
                    price,
                    price_change_percent,
                    volume,
                    high_24h,
                    low_24h,
                    prev_close_price,
                    timestamp
                ],
            )?;
            Ok(())
        }).await?;

        Ok(())
    }

    /// Store multiple price infos efficiently
    pub async fn store_price_infos(&self, price_infos: &[PriceInfo]) -> Result<(), Box<dyn std::error::Error>> {
        if price_infos.is_empty() {
            return Ok(());
        }

        // Clone the data to avoid lifetime issues
        let cloned_price_infos: Vec<PriceInfo> = price_infos.to_vec();

        self.conn.call(move |conn| {
            let tx = conn.transaction()?;

            for price_info in &cloned_price_infos {
                tx.execute(
                    "INSERT INTO prices (
                        symbol, price, price_change_percent, volume,
                        high_24h, low_24h, prev_close_price, timestamp
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, strftime('%s', 'now'))",
                    params![
                        price_info.symbol,
                        price_info.price,
                        price_info.price_change_percent,
                        price_info.volume,
                        price_info.high_24h,
                        price_info.low_24h,
                        price_info.prev_close_price
                    ],
                )?;
            }

            tx.commit()?;
            Ok(())
        }).await?;

        Ok(())
    }

    /// Store candle data
    pub async fn store_candles(&self, symbol: &str, timeframe: &str, candles: &[Candle]) -> Result<(), Box<dyn std::error::Error>> {
        if candles.is_empty() {
            return Ok(());
        }

        let symbol = symbol.to_string();
        let timeframe = timeframe.to_string();
        // Clone the candles to avoid lifetime issues
        let cloned_candles = candles.to_vec();

        self.conn.call(move |conn| {
            let tx = conn.transaction()?;

            for candle in &cloned_candles {
                tx.execute(
                    "INSERT INTO candles (
                        symbol, timeframe, open, high, low, close, volume, timestamp
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        &symbol,
                        &timeframe,
                        candle.open,
                        candle.high,
                        candle.low,
                        candle.close,
                        candle.volume,
                        candle.timestamp
                    ],
                )?;
            }

            tx.commit()?;
            Ok(())
        }).await?;

        Ok(())
    }

    /// Get latest price for a symbol
    pub async fn get_latest_price(&self, symbol: &str) -> Result<Option<PriceInfo>, Box<dyn std::error::Error>> {
        let symbol = symbol.to_string();

        let result = self.conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT symbol, price, price_change_percent, volume,
                        high_24h, low_24h, prev_close_price
                 FROM prices
                 WHERE symbol = ?
                 ORDER BY timestamp DESC
                 LIMIT 1"
            )?;

            let price_info = stmt.query_row(params![symbol], |row| {
                Ok(PriceInfo {
                    symbol: row.get(0)?,
                    price: row.get(1)?,
                    price_change_percent: row.get(2)?,
                    volume: row.get(3)?,
                    high_24h: row.get(4)?,
                    low_24h: row.get(5)?,
                    prev_close_price: row.get(6)?,
                })
            }).optional()?;

            Ok(price_info)
        }).await?;

        Ok(result)
    }

    /// Get candles for a symbol and timeframe within date range
    pub async fn get_candles(
        &self,
        symbol: &str,
        timeframe: &str,
        limit: usize
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
        let symbol = symbol.to_string();
        let timeframe = timeframe.to_string();

        let result = self.conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT open, high, low, close, volume, timestamp
                 FROM candles
                 WHERE symbol = ? AND timeframe = ?
                 ORDER BY timestamp DESC
                 LIMIT ?"
            )?;

            let mut candles = Vec::new();
            let mut rows = stmt.query_map(params![symbol, timeframe, limit as i64], |row| {
                Ok(Candle {
                    open: row.get(0)?,
                    high: row.get(1)?,
                    low: row.get(2)?,
                    close: row.get(3)?,
                    volume: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })?;

            while let Some(candle) = rows.next() {
                candles.push(candle?);
            }

            // Reverse to get chronological order
            candles.reverse();

            Ok(candles)
        }).await?;

        Ok(result)
    }

    /// Get all symbols that have recent price data
    pub async fn get_active_symbols(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let result = self.conn.call(|conn| {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT symbol FROM prices
                 WHERE timestamp > strftime('%s', 'now', '-1 hour')
                 ORDER BY symbol"
            )?;

            let mut symbols = Vec::new();
            let mut rows = stmt.query_map([], |row| {
                let symbol: String = row.get(0)?;
                Ok(symbol)
            })?;

            while let Some(symbol) = rows.next() {
                symbols.push(symbol?);
            }

            Ok(symbols)
        }).await?;

        Ok(result)
    }

    /// Update sync metadata
    pub async fn update_sync_metadata(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let key = key.to_string();
        let value = value.to_string();

        self.conn.call(move |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO sync_metadata (key, value, updated_at)
                 VALUES (?, ?, strftime('%s', 'now'))",
                params![key, value],
            )?;
            Ok(())
        }).await?;

        Ok(())
    }

    /// Get sync metadata
    pub async fn get_sync_metadata(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let key = key.to_string();

        let result = self.conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT value FROM sync_metadata WHERE key = ?"
            )?;

            let value = stmt.query_row(params![key], |row| {
                let value: String = row.get(0)?;
                Ok(value)
            }).optional()?;

            Ok(value)
        }).await?;

        Ok(result)
    }

    /// Clean old data (keep last 30 days for prices, last 90 days for candles)
    pub async fn cleanup_old_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.call(|conn| {
            // Clean old price data (keep 30 days)
            conn.execute(
                "DELETE FROM prices WHERE timestamp < strftime('%s', 'now', '-30 days')",
                [],
            )?;

            // Clean old candle data (keep 90 days)
            conn.execute(
                "DELETE FROM candles WHERE timestamp < strftime('%s', 'now', '-90 days')",
                [],
            )?;

            // Optimize database
            conn.execute("VACUUM", [])?;

            Ok(())
        }).await?;

        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats, Box<dyn std::error::Error>> {
        let result = self.conn.call(|conn| {
            let price_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM prices",
                [],
                |row| row.get(0),
            )?;

            let candle_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM candles",
                [],
                |row| row.get(0),
            )?;

            let db_size: i64 = conn.query_row(
                "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()",
                [],
                |row| row.get(0),
            )?;

            Ok(DatabaseStats {
                price_records: price_count,
                candle_records: candle_count,
                database_size_bytes: db_size,
            })
        }).await?;

        Ok(result)
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub price_records: i64,
    pub candle_records: i64,
    pub database_size_bytes: i64,
}

impl DatabaseStats {
    pub fn database_size_mb(&self) -> f64 {
        self.database_size_bytes as f64 / (1024.0 * 1024.0)
    }
}
