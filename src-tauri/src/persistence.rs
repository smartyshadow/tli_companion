//! Простая персистентность локальных кэшей (без базы/без supabase)
//!
//! Цель: чтобы цены, полученные из логов, переживали новые сессии и перезапуск приложения.
//! Безопасность: пишем только в data_local_dir()/tli-companion/, никаких произвольных путей.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::types::AppSettings;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricesCacheFile {
    pub version: u32,
    pub prices: HashMap<i64, PersistedPriceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedPriceEntry {
    pub price: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsFile {
    pub version: u32,
    pub settings: AppSettings,
}

fn app_data_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("tli-companion"))
}

fn prices_cache_path() -> Option<PathBuf> {
    app_data_dir().map(|d| d.join("prices_cache.json"))
}

fn settings_path() -> Option<PathBuf> {
    app_data_dir().map(|d| d.join("settings.json"))
}

pub fn load_prices_cache() -> io::Result<HashMap<i64, PersistedPriceEntry>> {
    let Some(path) = prices_cache_path() else {
        return Ok(HashMap::new());
    };
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let data = fs::read_to_string(&path)?;
    // v2 format
    if let Ok(parsed) = serde_json::from_str::<PricesCacheFile>(&data) {
        return Ok(parsed.prices);
    }

    // v1 legacy: game_id -> price (without timestamp)
    let legacy: HashMap<i64, f64> = serde_json::from_str(&data).unwrap_or_default();
    let now = Utc::now();
    Ok(legacy
        .into_iter()
        .filter(|(_, p)| p.is_finite() && *p > 0.0)
        .map(|(k, p)| (k, PersistedPriceEntry { price: p, updated_at: now }))
        .collect())
}

fn atomic_write(path: &Path, content: &str) -> io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir)?;

    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, content)?;
    // Windows: rename поверх существующего может падать, поэтому сначала удаляем старый.
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(tmp, path)?;
    Ok(())
}

pub fn save_prices_cache(prices: &HashMap<i64, PersistedPriceEntry>) -> io::Result<()> {
    let Some(path) = prices_cache_path() else {
        return Ok(());
    };

    // Фильтруем мусорные значения (на всякий случай)
    let mut sanitized: HashMap<i64, PersistedPriceEntry> = HashMap::new();
    for (k, v) in prices {
        if v.price.is_finite() && v.price > 0.0 {
            sanitized.insert(*k, v.clone());
        }
    }

    let file = PricesCacheFile {
        version: 2,
        prices: sanitized,
    };
    let json = serde_json::to_string(&file).unwrap_or_else(|_| "{\"version\":1,\"prices\":{}}".to_string());
    atomic_write(&path, &json)
}

pub fn load_settings() -> io::Result<Option<AppSettings>> {
    let Some(path) = settings_path() else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path)?;
    if let Ok(parsed) = serde_json::from_str::<SettingsFile>(&data) {
        return Ok(Some(parsed.settings));
    }

    // legacy: raw AppSettings without wrapper
    let legacy: AppSettings =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(legacy))
}

pub fn save_settings(settings: &AppSettings) -> io::Result<()> {
    let Some(path) = settings_path() else {
        return Ok(());
    };

    let file = SettingsFile {
        version: 1,
        settings: settings.clone(),
    };
    let json =
        serde_json::to_string(&file).unwrap_or_else(|_| "{\"version\":1,\"settings\":{}}".to_string());
    atomic_write(&path, &json)
}
