pub mod app;
pub mod binance;
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod database;
#[cfg(not(target_arch = "wasm32"))]
pub mod input;
#[cfg(not(target_arch = "wasm32"))]
pub mod theme;
#[cfg(not(target_arch = "wasm32"))]
pub mod ui;
pub mod utils;

// Web-specific modules
#[cfg(target_arch = "wasm32")]
pub mod web;
