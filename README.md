# 🪙 CoinPeek

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)
![License](https://img.shields.io/github/license/josiah-mbao/coinpeek?color=blue)
![Tests](https://img.shields.io/badge/tests-30%2F30-brightgreen)

A high-performance cryptocurrency price monitor built with Rust, featuring both terminal and web interfaces. Real-time price tracking from Binance API with persistent SQLite storage and comprehensive offline capabilities.

![coinpeek_demo](https://github.com/user-attachments/assets/d075dee7-08ec-4265-88f8-574b751b81aa)

## ✨ Features

- **Real-time Monitoring**: Live cryptocurrency prices via Binance API with configurable refresh intervals
- **Cross-Platform**: Native terminal TUI (ratatui) and web WASM (Yew) versions from single codebase
- **Persistent Storage**: SQLite database with efficient WAL mode and automatic data cleanup (30d prices, 90d candles)
- **Advanced Filtering**: Sort by symbol/price/change/volume, preset filters (gainers/losers/volatile), real-time search
- **Offline Resilience**: Graceful degradation with data freshness indicators and manual offline toggle
- **Price Alerts**: Configurable notifications for price thresholds with terminal bell alerts
- **Comprehensive Testing**: 30 unit tests covering critical paths for reliability

## 🏗️ Design Decisions

- **Async-First Architecture**: Tokio runtime enables concurrent API calls and responsive UI despite network operations
- **Efficient Persistence**: SQLite with strategic indexing and bulk transactions handles high-frequency data storage
- **Error Resilience**: Categorized error handling (network/API/database) with graceful degradation and recovery
- **Configuration-Driven**: JSON-based config with auto-generation ensures user customization without code changes
- **Performance Optimized**: Intelligent caching, async operations, and bulk processing maintain smooth 60fps interaction

## 📦 Installation

### Prerequisites
- Rust toolchain (≥1.70)
- Internet connection for live data

```bash
# Clone repository
git clone https://github.com/josiah-mbao/coinpeek.git
cd coinpeek

# Run tests
cargo test

# Run application (auto-creates config)
cargo run
```

## 🚀 Usage

### Configuration
Edit `coinpeek.json` (created automatically):

```json
{
  "symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT"],
  "refresh_interval_seconds": 5
}
```

### Controls

**Navigation**: `↑/↓` arrows, mouse click  
**Search**: `/` to enter search mode  
**Sorting**: `s` cycle modes, `d` toggle direction  
**Filtering**: `f` cycle presets, `c` clear filters  
**Alerts**: `Ctrl+A` for alert management  
**Offline**: `o` toggle offline mode  
**Help**: `?` show help, `q` quit  

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific modules
cargo test --test database_tests
cargo test --test api_tests
```

## 📝 License

MIT License. See [LICENSE](LICENSE) for details.
