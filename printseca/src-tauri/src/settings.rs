use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "settings.json";
const CONFIG_KEY: &str = "config";
const DAY_SECS: u64 = 86_400;

/// Configuração persistida do app.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    /// Intervalo entre impressões, em dias.
    pub interval_days: u32,
    /// "notify" (avisa) ou "auto" (imprime sozinho).
    pub mode: String,
    /// true = página colorida (CMYK), false = página preto e branco.
    pub color: bool,
    /// Nome da impressora; None = impressora padrão do sistema.
    pub printer: Option<String>,
    /// Timestamp (unix, segundos) da última impressão.
    pub last_print_ts: Option<u64>,
    /// Timestamp da última notificação enviada (para não notificar em excesso).
    pub last_notified_ts: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval_days: 10,
            mode: "notify".into(),
            color: true,
            printer: None,
            last_print_ts: None,
            last_notified_ts: None,
        }
    }
}

pub fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn load_config<R: Runtime>(app: &AppHandle<R>) -> Config {
    match app.store(STORE_FILE) {
        Ok(store) => store
            .get(CONFIG_KEY)
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save_config<R: Runtime>(app: &AppHandle<R>, cfg: &Config) {
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(CONFIG_KEY, json!(cfg));
        let _ = store.save();
    }
}

/// Timestamp em que a próxima impressão fica devida.
pub fn next_due_ts(cfg: &Config) -> Option<u64> {
    cfg.last_print_ts
        .map(|t| t + cfg.interval_days as u64 * DAY_SECS)
}

/// Dias restantes até a próxima impressão (negativo = atrasado).
pub fn days_left(cfg: &Config) -> i64 {
    match next_due_ts(cfg) {
        Some(due) => (due as i64 - now_ts() as i64).div_euclid(DAY_SECS as i64),
        None => 0,
    }
}

/// Se já passou do intervalo desde a última impressão.
pub fn is_due(cfg: &Config) -> bool {
    match cfg.last_print_ts {
        Some(t) => now_ts() >= t + cfg.interval_days as u64 * DAY_SECS,
        None => true,
    }
}
