//! Глобальное состояние приложения TLI Companion
//! 
//! Управляет состоянием сессии фарма, кэшем предметов и настройками.

use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use log::{info, debug};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use crate::types::{
    AppSettings, FarmSessionState, ItemInfo, SessionStats, 
    ItemDropEvent, MapChangeEvent, MapEventType, AggregatedDrop
};
use crate::log_parser::LogParser;
use crate::persistence;
use crate::auth::{AuthSession};

/// Глобальное состояние приложения
pub struct AppState {
    /// Настройки приложения
    pub settings: RwLock<AppSettings>,
    /// Текущая сессия фарма
    pub session: RwLock<FarmSessionState>,
    /// Кэш информации о предметах (game_id -> ItemInfo)
    pub items_cache: RwLock<HashMap<i64, ItemInfo>>,
    /// Кэш текущих цен (game_id -> price)
    pub prices_cache: RwLock<HashMap<i64, persistence::PersistedPriceEntry>>,
    /// Флаг подключения к серверу
    pub is_connected: RwLock<bool>,
    /// Путь к файлу логов
    pub log_path: RwLock<Option<String>>,
    /// Auth session (access token in-memory)
    pub auth_session: RwLock<Option<AuthSession>>,
    /// Cancel flag for in-progress OAuth login
    pub auth_oauth_cancel: RwLock<Option<Arc<AtomicBool>>>,
    /// Общий парсер логов (нужен, чтобы сбрасывать кэш слотов при старте сессии)
    pub log_parser: Arc<Mutex<LogParser>>,
}

const PRICE_TTL_SEC: i64 = 60 * 60; // 1 hour

impl AppState {
    /// Создать новое состояние
    pub fn new(log_parser: Arc<Mutex<LogParser>>) -> Self {
        Self {
            settings: RwLock::new(AppSettings::default()),
            session: RwLock::new(FarmSessionState::default()),
            items_cache: RwLock::new(HashMap::new()),
            prices_cache: RwLock::new(HashMap::new()),
            is_connected: RwLock::new(false),
            log_path: RwLock::new(None),
            auth_session: RwLock::new(None),
            auth_oauth_cancel: RwLock::new(None),
            log_parser,
        }
    }

    pub async fn set_auth_session(&self, session: Option<AuthSession>) {
        let mut s = self.auth_session.write().await;
        *s = session;
    }

    pub async fn get_auth_email(&self) -> Option<String> {
        let s = self.auth_session.read().await;
        s.as_ref().and_then(|x| x.user_email.clone())
    }

    pub async fn get_auth_user_id(&self) -> Option<String> {
        let s = self.auth_session.read().await;
        s.as_ref().and_then(|x| x.user_id.clone())
    }

    pub async fn is_logged_in(&self) -> bool {
        let s = self.auth_session.read().await;
        s.is_some()
    }

    /// Получить валидный access token (refresh при необходимости).
    pub async fn get_valid_access_token(
        &self,
        http: &reqwest::Client,
        cfg: &crate::supabase_sync::SupabaseConfig,
    ) -> Option<String> {
        // fast-path
        {
            let s = self.auth_session.read().await;
            if let Some(sess) = s.as_ref() {
                if Utc::now() < sess.expires_at {
                    return Some(sess.access_token.clone());
                }
            }
        }

        // refresh-path (requires refresh token from keychain)
        let refresh = crate::auth::load_refresh_token().ok().flatten()?;
        match crate::auth::refresh_access_token(http, cfg, &refresh).await {
            Ok(new_sess) => {
                let token = new_sess.access_token.clone();
                self.set_auth_session(Some(new_sess)).await;
                Some(token)
            }
            Err(_) => {
                // если refresh не удался — сбрасываем сессию
                self.set_auth_session(None).await;
                None
            }
        }
    }

    pub async fn load_settings_from_disk(&self) {
        match persistence::load_settings() {
            Ok(Some(settings)) => {
                let mut s = self.settings.write().await;
                *s = settings;
                debug!("Loaded settings from disk");
            }
            Ok(None) => {}
            Err(e) => {
                debug!("Failed to load settings from disk: {}", e);
            }
        }
    }

    pub async fn save_settings_to_disk(&self) {
        let s = self.settings.read().await;
        if let Err(e) = persistence::save_settings(&s) {
            debug!("Failed to save settings to disk: {}", e);
        }
    }

    pub async fn resolve_supabase_config(&self) -> Option<crate::supabase_sync::SupabaseConfig> {
        // For distributed builds, defaults are embedded in code (public anon key).
        // For dev/CI, env can override.
        Some(crate::supabase_sync::SupabaseConfig::from_env_or_compile()?)
    }
    
    /// Начать новую сессию фарма
    pub async fn start_session(&self, preset_id: Option<String>) {
        let mut session = self.session.write().await;
        *session = FarmSessionState {
            session_id: None,
            started_at: Some(Utc::now()),
            maps_completed: 0,
            total_duration_sec: 0,
            is_on_map: false,
            current_map_started: None,
            last_map_event_type: None,
            last_map_event_ts: None,
            last_map_scene: None,
            drops: HashMap::new(),
            preset_id,
        };
        info!("Farm session started");
    }
    
    /// Завершить сессию
    pub async fn end_session(&self) -> FarmSessionState {
        let session = self.session.read().await;
        let result = session.clone();
        drop(session);
        
        let mut session = self.session.write().await;
        *session = FarmSessionState::default();
        info!("Farm session ended");
        
        result
    }
    
    /// Обработать событие входа на карту
    pub async fn handle_map_enter(&self, ts: DateTime<Utc>) {
        let mut session = self.session.write().await;
        if session.started_at.is_none() {
            return;
        }

        // Дедуп: если уже на карте — ничего не делаем
        if session.is_on_map {
            return;
        }
        session.is_on_map = true;
        session.current_map_started = Some(ts);
        debug!("Entered map");
    }
    
    /// Обработать событие выхода с карты
    pub async fn handle_map_exit(&self, ts: DateTime<Utc>) {
        let mut session = self.session.write().await;
        if session.started_at.is_none() {
            return;
        }
        
        // Если EnterMap не был пойман (например, сессию начали уже внутри карты),
        // считаем, что карта началась в момент старта сессии.
        let map_started = session.current_map_started.or(session.started_at);

        // Увеличиваем счётчик карт всегда на ExitToHideout: это завершение карты по смыслу.
        session.maps_completed += 1;

        // Добавляем время карты к общему времени (map-only duration).
        if let Some(started) = map_started {
            let duration = (ts - started).num_seconds();
            if duration > 0 {
                session.total_duration_sec += duration as i32;
            }
        }
        
        session.is_on_map = false;
        session.current_map_started = None;
        debug!("Exited map, total maps: {}", session.maps_completed);
    }
    
    /// Обработать событие смены карты
    pub async fn handle_map_change(&self, event: &MapChangeEvent) {
        let mut session = self.session.write().await;
        if session.started_at.is_none() {
            return;
        }

        // 1) Жёсткая дедупликация: одинаковое событие по той же сцене, пришедшее почти сразу.
        if let (Some(last_ty), Some(last_ts), Some(last_scene)) = (
            &session.last_map_event_type,
            session.last_map_event_ts,
            &session.last_map_scene,
        ) {
            if last_ty == &event.event_type && last_scene == &event.scene_name {
                let dt = (event.timestamp - last_ts).num_seconds().abs();
                if dt <= 2 {
                    return;
                }
            }
        }

        // 2) Анти-дубль: два подряд Exit без Enter между ними — игнорируем второй и далее.
        if event.event_type == MapEventType::ExitToHideout
            && session.last_map_event_type == Some(MapEventType::ExitToHideout)
        {
            // Мы уже в “убежище” по логике сессии.
            session.last_map_event_ts = Some(event.timestamp);
            session.last_map_scene = Some(event.scene_name.clone());
            return;
        }

        match event.event_type {
            MapEventType::EnterMap => {
                if !session.is_on_map {
                    session.is_on_map = true;
                    session.current_map_started = Some(event.timestamp);
                }
            }
            MapEventType::ExitToHideout => {
                // Считаем карту завершённой. Если EnterMap не был пойман (старт сессии внутри карты),
                // считаем что карта началась в момент старта сессии.
                let map_started = session.current_map_started.or(session.started_at);
                session.maps_completed += 1;

                if let Some(started) = map_started {
                    let duration = (event.timestamp - started).num_seconds();
                    if duration > 0 {
                        session.total_duration_sec += duration as i32;
                    }
                }

                session.is_on_map = false;
                session.current_map_started = None;
            }
        }

        session.last_map_event_type = Some(event.event_type.clone());
        session.last_map_event_ts = Some(event.timestamp);
        session.last_map_scene = Some(event.scene_name.clone());
    }
    
    /// Добавить дроп
    pub async fn add_drop(&self, event: &ItemDropEvent) {
        let mut session = self.session.write().await;
        if session.started_at.is_none() {
            return;
        }
        
        let current = session.drops.get(&event.game_id).copied().unwrap_or(0);
        session.drops.insert(event.game_id, current + event.quantity);
        
        debug!("Added drop: game_id={}, qty={}, total={}", 
               event.game_id, event.quantity, current + event.quantity);
    }
    
    /// Обновить цену предмета в кэше
    pub async fn update_price(&self, game_id: i64, price: f64) {
        let mut prices = self.prices_cache.write().await;
        let now = Utc::now();
        prices.insert(game_id, persistence::PersistedPriceEntry { price, updated_at: now });
        debug!("Updated price: game_id={}, price={}", game_id, price);

        // Персистим на диск, чтобы цена переживала новую сессию/перезапуск.
        // Ошибки не фейлят приложение.
        let snapshot = prices.clone();
        drop(prices);
        if let Err(e) = persistence::save_prices_cache(&snapshot) {
            debug!("Failed to persist prices cache: {}", e);
        }
    }

    /// Загрузить кэш цен с диска (best-effort)
    pub async fn load_prices_cache_from_disk(&self) {
        match persistence::load_prices_cache() {
            Ok(map) => {
                let mut prices = self.prices_cache.write().await;
                // merge: не затираем уже обновлённые значения, если они есть
                for (k, v) in map {
                    prices.entry(k).or_insert(v);
                }
                debug!("Loaded prices cache from disk: {} items", prices.len());
            }
            Err(e) => {
                debug!("Failed to load prices cache: {}", e);
            }
        }
    }

    /// Слить remote цены (Supabase current prices) в локальный кэш.
    /// Не перетираем более свежие значения.
    pub async fn merge_remote_prices(&self, rows: Vec<(i64, f64, DateTime<Utc>)>) {
        let mut prices = self.prices_cache.write().await;
        let mut updated = 0usize;
        for (game_id, price, ts) in rows {
            if !price.is_finite() || price <= 0.0 {
                continue;
            }
            let replace = match prices.get(&game_id) {
                None => true,
                Some(existing) => ts > existing.updated_at,
            };
            if replace {
                prices.insert(game_id, persistence::PersistedPriceEntry { price, updated_at: ts });
                updated += 1;
            }
        }
        if updated > 0 {
            debug!("Merged remote prices: {} updated", updated);
        }
    }

    fn is_price_stale_internal(entry: &persistence::PersistedPriceEntry) -> bool {
        (Utc::now() - entry.updated_at).num_seconds() > PRICE_TTL_SEC
    }

    /// Цена для расчётов (None если устарела)
    pub async fn get_effective_price(&self, game_id: i64) -> Option<f64> {
        let prices = self.prices_cache.read().await;
        let entry = prices.get(&game_id)?;
        if Self::is_price_stale_internal(entry) {
            return None;
        }
        Some(entry.price)
    }
    
    /// Получить цену предмета
    pub async fn get_price(&self, game_id: i64) -> Option<f64> {
        let prices = self.prices_cache.read().await;
        prices.get(&game_id).map(|p| p.price)
    }
    
    /// Загрузить информацию о предметах в кэш
    pub async fn load_items_cache(&self, items: Vec<ItemInfo>) {
        let mut cache = self.items_cache.write().await;
        for item in items {
            cache.insert(item.game_id, item);
        }
        info!("Loaded {} items into cache", cache.len());
    }
    
    /// Получить информацию о предмете
    pub async fn get_item_info(&self, game_id: i64) -> Option<ItemInfo> {
        let cache = self.items_cache.read().await;
        cache.get(&game_id).cloned()
    }
    
    /// Получить статистику сессии
    pub async fn get_session_stats(&self) -> SessionStats {
        let session = self.session.read().await;
        let prices = self.prices_cache.read().await;
        
        let total_items: i32 = session.drops.values().sum();
        let unique_items = session.drops.len() as i32;
        
        // Вычисляем общую стоимость
        let mut total_value = 0.0;
        let mut stale_price_lines = 0i32;
        for (game_id, qty) in &session.drops {
            if let Some(price_entry) = prices.get(game_id) {
                // Доход считаем всегда (даже по устаревшим ценам), но помечаем что часть цен старые,
                // чтобы UI мог попросить пользователя обновить прайсчек.
                total_value += price_entry.price * (*qty as f64);
                if Self::is_price_stale_internal(price_entry) {
                    stale_price_lines += 1;
                }
            }
        }
        
        // Длительность сессии = wall-clock от started_at (а не только “на карте”),
        // иначе при проблемах с EnterMap таймер навсегда будет 0.
        let duration_sec = session
            .started_at
            .map(|started| (Utc::now() - started).num_seconds().max(0) as i32)
            .unwrap_or(0);

        // Средняя длительность карты: используем map-only время (total_duration_sec).
        // Если карт ещё нет, но мы на карте — показываем время текущей карты как “среднее” (удобно для первой карты).
        let current_map_elapsed_sec = session
            .current_map_started
            .map(|started| (Utc::now() - started).num_seconds().max(0) as i32)
            .unwrap_or(0);

        let avg_map_duration_sec = if session.maps_completed > 0 {
            // round к ближайшей секунде (чтобы не было систематического занижения)
            ((session.total_duration_sec as f64) / (session.maps_completed as f64)).round() as i32
        } else if session.is_on_map && current_map_elapsed_sec > 0 {
            current_map_elapsed_sec
        } else {
            0
        };
        
        // Доход в час
        let hourly_profit = if duration_sec > 0 {
            total_value / (duration_sec as f64) * 3600.0
        } else {
            0.0
        };
        
        SessionStats {
            total_items,
            unique_items,
            total_value,
            maps_completed: session.maps_completed,
            duration_sec,
            avg_map_duration_sec,
            stale_price_lines,
            hourly_profit,
        }
    }
    
    /// Получить агрегированные дропы для отображения
    pub async fn get_aggregated_drops(&self) -> Vec<AggregatedDrop> {
        let session = self.session.read().await;
        let items_cache = self.items_cache.read().await;
        let prices = self.prices_cache.read().await;
        
        let mut drops: Vec<AggregatedDrop> = session.drops.iter().map(|(game_id, qty)| {
            let item_info = items_cache.get(game_id).cloned();
            let (unit_price, price_updated_at, price_is_stale) = match prices.get(game_id) {
                Some(p) => (p.price, Some(p.updated_at), Self::is_price_stale_internal(p)),
                None => (0.0, None, false),
            };
            let total_value = unit_price * (*qty as f64);
            
            AggregatedDrop {
                game_id: *game_id,
                item_info,
                quantity: *qty,
                total_value,
                unit_price,
                price_updated_at,
                price_is_stale,
            }
        }).collect();
        
        // Сортируем по стоимости (от большей к меньшей)
        drops.sort_by(|a, b| b.total_value.partial_cmp(&a.total_value).unwrap_or(std::cmp::Ordering::Equal));
        
        drops
    }
    
    /// Проверить, активна ли сессия
    pub async fn is_session_active(&self) -> bool {
        let session = self.session.read().await;
        session.started_at.is_some()
    }
    
    /// Установить путь к логам (и сохранить в настройки)
    pub async fn set_log_path(&self, path: Option<String>) {
        let mut log_path = self.log_path.write().await;
        *log_path = path.clone();
        
        // Сохраняем в настройки для персистентности
        let mut settings = self.settings.write().await;
        settings.custom_log_path = path;
        if let Err(e) = persistence::save_settings(&settings) {
            log::warn!("Failed to save settings with custom log path: {}", e);
        }
    }
    
    /// Получить путь к логам
    pub async fn get_log_path(&self) -> Option<String> {
        let log_path = self.log_path.read().await;
        log_path.clone()
    }
    
    /// Получить custom_log_path из настроек
    pub async fn get_custom_log_path(&self) -> Option<String> {
        let settings = self.settings.read().await;
        settings.custom_log_path.clone()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(LogParser::new())))
    }
}
