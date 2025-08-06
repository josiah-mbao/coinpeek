# 🪙 CoinPeek

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)
![License](https://img.shields.io/github/license/josiah-mbao/coinpeek?color=blue)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![Refresh](https://img.shields.io/badge/refresh_interval-5s-blue)
![Binance API](https://img.shields.io/badge/API-Binance-yellow)

**CoinPeek** is a Rust-powered terminal application that displays real-time cryptocurrency prices using the Binance API. Built for speed and clarity, it turns your terminal into a sleek, auto-refreshing crypto dashboard.


![coinpeek_demo](https://github.com/user-attachments/assets/d075dee7-08ec-4265-88f8-574b751b81aa)


 ⚡ Fast. 🔁 Real-time. 🖥️ Terminal-native.

---

## ✨ Features

- 📈 Live crypto prices with Binance API
- 🖥️ Minimalist terminal UI powered by [`ratatui`](https://github.com/ratatui-org/ratatui)
- 🔁 Auto-refreshes every 5 seconds
- 🔧 Easily customize the coin symbols (BTCUSDT, ETHUSDT, etc.)
- 🦀 100% Rust — fast, safe, and clean

---

## 📦 Built With

- [`ratatui`](https://github.com/ratatui-org/ratatui) — for terminal UI rendering
- [`crossterm`](https://github.com/crossterm-rs/crossterm) — for terminal backend
- [`reqwest`](https://github.com/seanmonstar/reqwest) — to fetch API data
- [`serde`](https://github.com/serde-rs/serde) — to deserialize JSON responses

---

## 🚀 Getting Started

### Prerequisites

- Rust (recommended: `>= 1.70`)
- Internet connection (Binance API requires it)

### Installation

```bash
# Clone the repository
git clone https://github.com/josiah-mbao/coinpeek.git
cd coinpeek

# Run the application
cargo run
```

### Next Steps
- Add support for more exchanges
- Enable price change highlights (color-coded)
- Config file support for custom refresh intervals and coins

### Author
Built with focus and curiousity by @josiah-mbao

Proverbs 16:3
