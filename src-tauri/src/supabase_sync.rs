//! Supabase sync (prices)
//!
//! - Public read: fetch current prices from tli_current_prices (anon)
//! - Optional write: send raw samples to RPC upsert_market_price (requires user JWT)
//!
//! Config via env:
//! - VITE_SUPABASE_URL
//! - VITE_SUPABASE_ANON_KEY

use chrono::{DateTime, Utc};
use log::{debug, warn};
use serde::Deserialize;
use crate::supabase_defaults;
use crate::types::ItemInfo;

#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
}

impl SupabaseConfig {
    pub fn from_env_or_compile() -> Option<Self> {
        let url = std::env::var("VITE_SUPABASE_URL")
            .ok()
            .or_else(|| option_env!("VITE_SUPABASE_URL").map(|s| s.to_string()))
            .unwrap_or_else(|| supabase_defaults::SUPABASE_URL.to_string());
        let anon_key = std::env::var("VITE_SUPABASE_ANON_KEY")
            .ok()
            .or_else(|| option_env!("VITE_SUPABASE_ANON_KEY").map(|s| s.to_string()))
            .unwrap_or_else(|| supabase_defaults::SUPABASE_ANON_KEY.to_string());
        Some(Self { url, anon_key })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CurrentPriceRow {
    game_id: i64,
    price: f64,
    last_updated: DateTime<Utc>,
}

pub async fn fetch_current_prices(
    client: &reqwest::Client,
    cfg: &SupabaseConfig,
) -> Result<Vec<(i64, f64, DateTime<Utc>)>, String> {
    let endpoint = format!(
        "{}/rest/v1/tli_current_prices?select=game_id,price,last_updated",
        cfg.url.trim_end_matches('/')
    );

    let resp = client
        .get(endpoint)
        .header("apikey", &cfg.anon_key)
        .header("Authorization", format!("Bearer {}", cfg.anon_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Supabase fetch_current_prices failed: {}", resp.status()));
    }

    let rows: Vec<CurrentPriceRow> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| (r.game_id, r.price, r.last_updated))
        .collect())
}

pub async fn upsert_market_price(
    client: &reqwest::Client,
    cfg: &SupabaseConfig,
    user_jwt: &str,
    game_id: i64,
    prices: &[f64],
    currency_id: i64,
) -> Result<(), String> {
    if prices.is_empty() {
        return Ok(());
    }

    let endpoint = format!(
        "{}/rest/v1/rpc/upsert_market_price",
        cfg.url.trim_end_matches('/')
    );

    let body = serde_json::json!({
        "p_game_id": game_id,
        "p_prices": prices,
        "p_currency_id": currency_id
    });

    let resp = client
        .post(endpoint)
        .header("apikey", &cfg.anon_key)
        .header("Authorization", format!("Bearer {}", user_jwt))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Supabase upsert_market_price failed: {} {}", status, text));
    }

    Ok(())
}

pub fn warn_if_missing() {
    if SupabaseConfig::from_env_or_compile().is_none() {
        warn!("Supabase env vars missing (VITE_SUPABASE_URL / VITE_SUPABASE_ANON_KEY). Price sync disabled.");
    } else {
        debug!("Supabase env vars present. Price sync enabled.");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Game Items (names, categories, icons)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct GameItemRow {
    game_id: i64,
    name_en: Option<String>,
    name_ru: Option<String>,
    name_cn: Option<String>,
    category: Option<String>,
    icon_url: Option<String>,
}

/// Fetch all game items from Supabase (public read, anon key)
pub async fn fetch_game_items(
    client: &reqwest::Client,
    cfg: &SupabaseConfig,
) -> Result<Vec<ItemInfo>, String> {
    let endpoint = format!(
        "{}/rest/v1/tli_game_items?select=game_id,name_en,name_ru,name_cn,category,icon_url",
        cfg.url.trim_end_matches('/')
    );

    let resp = client
        .get(endpoint)
        .header("apikey", &cfg.anon_key)
        .header("Authorization", format!("Bearer {}", cfg.anon_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Supabase fetch_game_items failed: {}", resp.status()));
    }

    let rows: Vec<GameItemRow> = resp.json().await.map_err(|e| e.to_string())?;
    
    Ok(rows
        .into_iter()
        .map(|r| ItemInfo {
            game_id: r.game_id,
            name: r.name_en.clone().unwrap_or_else(|| format!("ID: {}", r.game_id)),
            name_en: r.name_en,
            name_ru: r.name_ru,
            name_cn: r.name_cn,
            category: r.category.unwrap_or_else(|| "unknown".to_string()),
            icon_url: r.icon_url,
        })
        .collect())
}
