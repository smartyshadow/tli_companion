//! Tauri Commands для IPC с frontend
//! 
//! Эти функции доступны из JavaScript/TypeScript через Tauri invoke.

use tauri::State;
use std::sync::Arc;
use log::info;

use crate::state::AppState;
use crate::types::{SessionStats, AggregatedDrop, AppSettings, ItemInfo, UserProfile};
use crate::file_watcher::find_log_path;
use std::sync::atomic::AtomicBool;

/// Найти путь к файлу логов автоматически
#[tauri::command]
pub async fn find_log_file() -> Result<Option<String>, String> {
    info!("Looking for TLI log file...");
    
    match find_log_path() {
        Some(path) => {
            let path_str = path.to_string_lossy().to_string();
            info!("Found log file: {}", path_str);
            Ok(Some(path_str))
        }
        None => {
            info!("Log file not found automatically");
            Ok(None)
        }
    }
}

/// Установить путь к файлу логов вручную
#[tauri::command]
pub async fn set_log_path(
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<bool, String> {
    // Проверяем, существует ли файл
    if !std::path::Path::new(&path).exists() {
        return Err("File does not exist".to_string());
    }

    // Безопасность: по умолчанию разрешаем только UE_game.log, чтобы не дать приложению
    // читать произвольные файлы пользователя через IPC.
    if let Some(file_name) = std::path::Path::new(&path).file_name().and_then(|n| n.to_str()) {
        if !file_name.eq_ignore_ascii_case("UE_game.log") {
            return Err("Only UE_game.log is supported for security reasons".to_string());
        }
    } else {
        return Err("Invalid path".to_string());
    }
    
    state.set_log_path(Some(path.clone())).await;
    info!("Log path set to: {}", path);
    Ok(true)
}

/// Начать новую сессию фарма
#[tauri::command]
pub async fn start_session(
    state: State<'_, Arc<AppState>>,
    preset_id: Option<String>,
) -> Result<(), String> {
    state.start_session(preset_id).await;
    Ok(())
}

/// Завершить сессию фарма
#[tauri::command]
pub async fn end_session(
    state: State<'_, Arc<AppState>>,
) -> Result<SessionStats, String> {
    // ВАЖНО: сначала берём финальные stats, потом сбрасываем состояние.
    // Иначе get_session_stats() вернёт нули, потому что session уже reset.
    let stats = state.get_session_stats().await;
    let _session_data = state.end_session().await;
    Ok(stats)
}

/// Получить текущую статистику сессии
#[tauri::command]
pub async fn get_session_stats(
    state: State<'_, Arc<AppState>>,
) -> Result<SessionStats, String> {
    Ok(state.get_session_stats().await)
}

/// Получить список дропов за сессию
#[tauri::command]
pub async fn get_drops(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<AggregatedDrop>, String> {
    Ok(state.get_aggregated_drops().await)
}

/// Проверить, активна ли сессия
#[tauri::command]
pub async fn is_session_active(
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    Ok(state.is_session_active().await)
}

/// Получить настройки приложения
#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<AppSettings, String> {
    let settings = state.settings.read().await;
    Ok(settings.clone())
}

/// Сохранить настройки приложения
#[tauri::command]
pub async fn save_settings(
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<(), String> {
    let mut current = state.settings.write().await;
    *current = settings;
    info!("Settings saved");
    drop(current);
    state.save_settings_to_disk().await;
    Ok(())
}

/// Получить информацию о предмете по game_id
#[tauri::command]
pub async fn get_item_info(
    state: State<'_, Arc<AppState>>,
    game_id: i64,
) -> Result<Option<ItemInfo>, String> {
    Ok(state.get_item_info(game_id).await)
}

/// Загрузить кэш предметов
#[tauri::command]
pub async fn load_items_cache(
    state: State<'_, Arc<AppState>>,
    items: Vec<ItemInfo>,
) -> Result<(), String> {
    state.load_items_cache(items).await;
    Ok(())
}

/// Обновить цену предмета
#[tauri::command]
pub async fn update_item_price(
    state: State<'_, Arc<AppState>>,
    game_id: i64,
    price: f64,
) -> Result<(), String> {
    state.update_price(game_id, price).await;
    Ok(())
}

/// Получить путь к логам
#[tauri::command]
pub async fn get_log_path(
    state: State<'_, Arc<AppState>>,
) -> Result<Option<String>, String> {
    Ok(state.get_log_path().await)
}

/// Получить версию приложения
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Открыть URL в браузере
#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct AuthStatus {
    pub is_logged_in: bool,
    pub email: Option<String>,
}

/// Получить статус авторизации
#[tauri::command]
pub async fn auth_status(state: State<'_, Arc<AppState>>) -> Result<AuthStatus, String> {
    let is_logged_in = state.is_logged_in().await;
    let email = state.get_auth_email().await;
    Ok(AuthStatus {
        is_logged_in,
        email,
    })
}

/// Войти через kripika.com (единая точка входа)
#[tauri::command]
pub async fn auth_sign_in_kripika(state: State<'_, Arc<AppState>>) -> Result<AuthStatus, String> {
    let cfg = state
        .resolve_supabase_config()
        .await
        .ok_or_else(|| "Supabase config missing".to_string())?;
    let http = reqwest::Client::new();
    let api_url = {
        let s = state.settings.read().await;
        s.api_url.clone()
    };

    // Cancel handle for this login attempt (used to free the port on user cancel).
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut lock = state.auth_oauth_cancel.write().await;
        *lock = Some(cancel.clone());
    }

    let sess = crate::auth::sign_in_via_kripika(&http, &cfg, &api_url, cancel.clone()).await?;
    state.set_auth_session(Some(sess)).await;

    // Clear cancel flag after success
    {
        let mut lock = state.auth_oauth_cancel.write().await;
        *lock = None;
    }
    auth_status(state).await
}

/// Отменить текущий процесс OAuth (освобождает порт 49733 сразу)
#[tauri::command]
pub async fn auth_cancel_login(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let opt = { state.auth_oauth_cancel.read().await.clone() };
    if let Some(flag) = opt {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    let mut lock = state.auth_oauth_cancel.write().await;
    *lock = None;
    Ok(())
}

/// Выйти — удаляем refresh token из keychain и очищаем in-memory сессию
#[tauri::command]
pub async fn auth_sign_out(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    crate::auth::clear_refresh_token()?;
    state.set_auth_session(None).await;
    Ok(())
}

#[derive(serde::Deserialize)]
struct AuthUserResponse {
    id: String,
    email: Option<String>,
}

/// Получить профиль пользователя из public.profiles (kripika.com)
#[tauri::command]
pub async fn get_my_profile(state: State<'_, Arc<AppState>>) -> Result<Option<UserProfile>, String> {
    let cfg = state
        .resolve_supabase_config()
        .await
        .ok_or_else(|| "Supabase config missing".to_string())?;

    let http = reqwest::Client::new();
    let jwt = state
        .get_valid_access_token(&http, &cfg)
        .await
        .ok_or_else(|| "Not logged in".to_string())?;

    // Prefer user_id from auth session; fallback to /auth/v1/user (paranoia).
    let mut user_id = state.get_auth_user_id().await;
    if user_id.is_none() {
        let endpoint = format!("{}/auth/v1/user", cfg.url.trim_end_matches('/'));
        let resp = http
            .get(endpoint)
            .header("apikey", &cfg.anon_key)
            .header("Authorization", format!("Bearer {}", jwt))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            if let Ok(u) = resp.json::<AuthUserResponse>().await {
                user_id = Some(u.id);
                // Patch in-memory session so subsequent calls have user_id/email.
                let mut lock = state.auth_session.write().await;
                if let Some(sess) = lock.as_mut() {
                    sess.user_id = user_id.clone();
                    sess.user_email = u.email;
                }
            }
        }
    }

    let user_id = user_id.ok_or_else(|| "Missing user id".to_string())?;

    let endpoint = format!(
        "{}/rest/v1/profiles?id=eq.{}&select=id,username,display_name,avatar_url,level,total_xp",
        cfg.url.trim_end_matches('/'),
        user_id
    );

    let resp = http
        .get(endpoint)
        .header("apikey", &cfg.anon_key)
        .header("Authorization", format!("Bearer {}", jwt))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Profile fetch failed: {} {}", status, text));
    }

    let rows: Vec<UserProfile> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().next())
}
