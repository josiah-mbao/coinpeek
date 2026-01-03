# 🪙 CoinPeek

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)
![License](https://img.shields.io/github/license/josiah-mbao/coinpeek?color=blue)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![Tests](https://img.shields.io/badge/tests-30%2F30-brightgreen)
![Database](https://img.shields.io/badge/database-SQLite-blue)
![Binance API](https://img.shields.io/badge/API-Binance-yellow)

**CoinPeek** is a high-performance, production-ready terminal application for real-time cryptocurrency price monitoring. Built with Rust, it combines a sleek TUI interface with robust data persistence, comprehensive testing, and enterprise-grade architecture.

![coinpeek_demo](https://github.com/user-attachments/assets/d075dee7-08ec-4265-88f8-574b751b81aa)

 ⚡ **Fast**. 🔄 **Real-time**. 💾 **Persistent**. 🧪 **Tested**. 🖥️ **Terminal-native**.

---

## ✨ Features

### **Core Functionality**
- 📊 **Real-time Price Monitoring** - Live cryptocurrency prices via Binance API
- 💾 **Persistent Data Storage** - SQLite database with automatic data retention
- 🖥️ **Minimalist TUI** - Clean terminal interface powered by `ratatui`
- 🔄 **Auto-refresh** - Configurable update intervals with intelligent caching
- 📈 **Multi-sort Options** - Sort by symbol, price, 24h change, or volume
- 🎯 **Interactive Navigation** - Keyboard-driven selection and browsing

### **Production Features**
- 🧪 **Comprehensive Testing** - 30 unit tests covering all critical paths
- 🔧 **Configuration Management** - JSON-based config with hot-reload capability
- 🛡️ **Error Resilience** - Graceful handling of network failures and API issues
- 📊 **Performance Monitoring** - Database statistics and health metrics
- 🔄 **Offline Capability** - Full offline operation with data freshness indicators

### **Offline Awareness**
- 🟢 **🟢 Online Mode** - Real-time sync with green status indicator
- 🟡 **🟡 Degraded Mode** - Network issues with yellow warning
- 🔴 **🔴 Offline Mode** - Manual offline toggle with red indicator
- 📊 **Data Age Display** - Shows "2m ago", "1h ago", etc.
- 💾 **Persistent Cache** - 30 days price data, 90 days candle data

---

## 🏗️ Architecture & Design Decisions

### **System Architecture**

CoinPeek follows a **layered architecture** with clear separation of concerns:

```
┌─────────────────┐
│   Presentation  │  ← TUI, User Input, Display Logic
│     Layer       │
├─────────────────┤
│  Application    │  ← Business Logic, State Management
│     Layer       │
├─────────────────┤
│   Data Access   │  ← Database Operations, API Calls
│     Layer       │
├─────────────────┤
│  Infrastructure │  ← SQLite, HTTP Client, Config
│     Layer       │
└─────────────────┘
```

### **Key Design Decisions**

#### **1. SQLite Database Choice**
**Decision**: Use SQLite for data persistence instead of in-memory storage.

**Rationale**:
- **Reliability**: Data survives application restarts and crashes
- **Performance**: WAL mode enables concurrent reads/writes
- **Scalability**: Handles historical data storage efficiently
- **Zero Configuration**: No external database server required
- **ACID Compliance**: Ensures data integrity during operations

**Implementation**:
- Optimized with 1GB cache and WAL mode
- Strategic indexing on symbol and timestamp columns
- Automatic cleanup of old data (30 days prices, 90 days candles)

#### **2. Async-First Architecture**
**Decision**: Build with Tokio async runtime from the ground up.

**Rationale**:
- **Scalability**: Non-blocking I/O for concurrent API calls
- **Performance**: Efficient resource utilization
- **Future-Proofing**: Ready for WebSocket real-time feeds
- **Ecosystem**: Rich async ecosystem for database and HTTP operations

**Implementation**:
- `tokio-rusqlite` for async database operations
- `reqwest` with async HTTP client
- Structured concurrency with proper error handling

#### **3. Repository Pattern for Data Access**
**Decision**: Implement repository pattern for database operations.

**Rationale**:
- **Testability**: Easy to mock and test data operations
- **Maintainability**: Clear separation between business logic and data access
- **Flexibility**: Can swap storage backends without changing business logic
- **Consistency**: Standardized CRUD operations across entities

**Implementation**:
- Clean async methods for all database operations
- Transaction-based bulk operations for efficiency
- Comprehensive error handling and logging

#### **4. Comprehensive Testing Strategy**
**Decision**: Implement thorough unit testing with 100% critical path coverage.

**Rationale**:
- **Reliability**: Catch regressions before they reach production
- **Documentation**: Tests serve as living documentation
- **Confidence**: Safe refactoring and feature additions
- **Quality**: Enterprise-grade code quality standards

**Test Coverage**:
- **Database Layer** (9 tests): CRUD operations, transactions, edge cases
- **API Layer** (10 tests): Parsing, validation, error handling
- **Application Layer** (11 tests): Business logic, state management, UI

#### **5. Configuration-Driven Design**
**Decision**: JSON-based configuration with automatic generation.

**Rationale**:
- **User-Friendly**: Easy to modify without code changes
- **Version Control**: Config files can be tracked and versioned
- **Flexibility**: Runtime behavior modification
- **Defaults**: Sensible defaults for new users

**Features**:
- Auto-generation on first run
- Hot-reload capability
- Validation and error handling
- Comprehensive documentation

#### **6. Error Handling Philosophy**
**Decision**: Fail gracefully with comprehensive error recovery.

**Rationale**:
- **User Experience**: Application continues working despite failures
- **Debugging**: Detailed error logging for troubleshooting
- **Resilience**: Network failures don't crash the application
- **Monitoring**: Error tracking for system health

**Implementation**:
- Result types throughout the codebase
- Graceful degradation (cached data fallback)
- User-friendly error messages
- Comprehensive logging

#### **7. Performance Optimizations**
**Decision**: Optimize for speed and efficiency at every level.

**Rationale**:
- **User Experience**: Instant startup and responsive interface
- **Resource Usage**: Minimal memory and CPU footprint
- **Scalability**: Handles growing datasets efficiently
- **Responsiveness**: Real-time updates without blocking UI

**Optimizations**:
- Database query optimization with strategic indexing
- Bulk operations for efficient data processing
- Intelligent caching with cache-first loading
- Async operations to prevent UI blocking

---

## 📦 Technical Stack

### **Core Dependencies**
- **`ratatui`** — Modern terminal UI framework
- **`tokio`** — Async runtime for concurrent operations
- **`rusqlite`** + **`tokio-rusqlite`** — SQLite database with async support
- **`reqwest`** — HTTP client for API calls
- **`serde`** — JSON serialization/deserialization
- **`chrono`** — Date/time handling

### **Testing Dependencies**
- **`tokio-test`** — Async testing utilities
- **`tempfile`** — Isolated database testing
- **`mockito`** — API mocking for integration tests

### **Development Tools**
- **Cargo** — Rust package manager and build system
- **Clippy** — Code linting and style checking
- **Rustfmt** — Code formatting
- **Comprehensive test suite** — 30 automated tests

---

## 🚀 Getting Started

### **Prerequisites**
- Rust toolchain (`>= 1.70`)
- Internet connection (for live data)
- Terminal with Unicode support

### **Installation**

```bash
# Clone the repository
git clone https://github.com/josiah-mbao/coinpeek.git
cd coinpeek

# Run tests to verify everything works
cargo test

# Run the application (creates config file automatically)
cargo run
```

### **Configuration**

CoinPeek uses a `coinpeek.json` file for customization. Created automatically on first run:

```json
{
  "symbols": [
    "BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT",
    "SOLUSDT", "DOTUSDT", "DOGEUSDT", "AVAXUSDT",
    "LTCUSDT", "LINKUSDT"
  ],
  "refresh_interval_seconds": 5
}
```

**Configuration Options**:
- `symbols`: Array of cryptocurrency pairs to track
- `refresh_interval_seconds`: Update frequency (default: 5 seconds)

### **Usage**

```bash
# Run with default configuration
cargo run

# Run tests
cargo test

# Build optimized release
cargo build --release
```

**Keyboard Controls**:

**Navigation:**
- `↑/↓` — Navigate between cryptocurrencies
- `?` — Show help screen
- `q` or `Ctrl+C` — Quit

**Sorting:**
- `s` — Cycle sort mode (Symbol → Price → Change % → Volume)
- `d` — Toggle sort direction (↑ Ascending ↔ ↓ Descending)

**Filtering:**
- `f` — Cycle filter presets (All → Top Gainers → Top Losers → High Volume → Volatile → Stable)
- `c` — Clear all filters and presets

**Data & Offline Mode:**
- `r` — Manual refresh
- `o` — Toggle offline mode (🟢 online ↔ 🔴 offline)
- `p` — Toggle pause/resume

---

## 🧪 Testing

CoinPeek maintains **30 comprehensive unit tests** covering all critical functionality:

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test --test database_tests
cargo test --test api_tests
cargo test --test app_tests

# Run with detailed output
cargo test -- --nocapture
```

**Test Coverage**:
- ✅ Database operations and data integrity
- ✅ API parsing and error handling
- ✅ Business logic and state management
- ✅ UI navigation and sorting
- ✅ Edge cases and error recovery

---

## 🔧 Development

### **Project Structure**
```
coinpeek/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   ├── app.rs           # Core application logic
│   ├── binance.rs       # API client and data models
│   ├── database.rs      # SQLite data access layer
│   ├── ui.rs            # Terminal UI rendering
│   ├── config.rs        # Configuration management
│   ├── input.rs         # Input handling
│   ├── theme.rs         # UI theming
│   └── utils.rs         # Utility functions
├── tests/
│   ├── database_tests.rs # Database layer tests
│   ├── api_tests.rs      # API layer tests
│   └── app_tests.rs      # Application layer tests
├── Cargo.toml           # Dependencies and metadata
├── coinpeek.json        # Configuration file
└── README.md           # This file
```

### **Contributing**
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

### **Code Quality**
- **Clippy**: `cargo clippy` for linting
- **Formatting**: `cargo fmt` for code style
- **Testing**: `cargo test` for validation
- **Documentation**: Comprehensive inline documentation

---

## 📊 Performance Characteristics

- **Startup Time**: Instant (loads from SQLite cache)
- **Memory Usage**: Minimal (~10MB resident)
- **Database Size**: Scales efficiently with data
- **API Calls**: Optimized with intelligent caching
- **UI Responsiveness**: 60fps smooth interaction

---

## 🔮 Roadmap

### **Phase 2: Real-time Feeds**
- WebSocket connections for live price updates
- Order book depth visualization
- Real-time trade feed

### **Phase 3: Advanced Features**
- Rate limiting and request management
- Multi-exchange support
- Price alerts and notifications

### **Phase 4: Offline & Sync**
- Offline mode with data synchronization
- Historical data analysis
- Portfolio tracking

---

## 📝 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

Built with focus and curiosity by [@josiah-mbao](https://github.com/josiah-mbao)

*"Commit to the LORD whatever you do, and he will establish your plans."* — Proverbs 16:3

---

**CoinPeek** — Professional cryptocurrency monitoring, built with Rust. 🚀
