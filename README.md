# 🪙 coinpeek

**coinpeek** is a Rust-powered terminal application that displays real-time cryptocurrency prices using the Binance API. Built for speed and clarity, it turns your terminal into a minimal, refreshing crypto dashboard.

![screenshot-placeholder](https://user-images.githubusercontent.com/your-username/screenshot.gif)

---

## ✨ Features

- 🖥️ Terminal UI (TUI) with clean, auto-refreshing layout
- 🔁 Real-time data pulled from Binance API
- ⚙️ Customizable list of crypto symbols (e.g. BTCUSDT, ETHUSDT)
- 🦀 Fast, efficient, and written in 100% Rust

---

## 📦 Built With

- [ratatui](https://github.com/ratatui-org/ratatui) – terminal user interface rendering
- [crossterm](https://github.com/crossterm-rs/crossterm) – terminal backend
- [reqwest](https://github.com/seanmonstar/reqwest) – HTTP client
- [serde](https://github.com/serde-rs/serde) – JSON parsing

---

## 🚀 Getting Started

### Prerequisites

- Rust (recommended version: `>=1.70`)
- Internet connection (to fetch price data)

### Installation

```bash
# Clone the repository
git clone https://github.com/josiah-mbao/coinpeek.git
cd coinpeek

# Run the project
cargo run
```

###  Preview
+-----------------------------------+
|         🪙 coinpeek               |
+-----------------------------------+
| BTC/USDT       $29,430.23         |
| ETH/USDT       $1,854.60          |
| DOGE/USDT      $0.062             |
+-----------------------------------+
| Last updated: 2025-07-21 12:03PM  |
+-----------------------------------+

### Author
Built with focus and curiousity by @josiah-mbao

Proverbs 16:3
