//! TLI Companion Library
//! 
//! Библиотека для парсинга логов Torchlight Infinite.

pub mod types;
pub mod log_parser;
pub mod file_watcher;
pub mod state;
pub mod commands;
pub mod persistence;
pub mod supabase_sync;
pub mod auth;
pub mod supabase_defaults;

pub use types::*;
pub use log_parser::LogParser;
pub use file_watcher::{LogWatcher, find_log_path};
pub use state::AppState;
