# 🪙 CoinPeek

**CoinPeek** is a Rust-powered terminal application that displays real-time cryptocurrency prices using the Binance API. Built for speed and clarity, it turns your terminal into a sleek, auto-refreshing crypto dashboard.

https://github.com/user-attachments/assets/c9482e47-3cd4-48e5-a68a-ed6d07345ee0

> ⚡ Fast. 🔁 Real-time. 🖥️ Terminal-native.

<p align="center">
  <a href="https://github.com/josiah-mbao/coinpeek/releases">
    <img src="https://img.shields.io/badge/version-0.1.0-blue.svg" alt="Version">
  </a>
  <a href="https://www.rust-lang.org">
    <img src="https://img.shields.io/badge/Rust-1.70+-orange.svg" alt="Rust Version">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="MIT License">
  </a>
</p>

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
