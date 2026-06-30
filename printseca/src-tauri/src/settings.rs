// ============================================================================
// settings.rs — Configuração persistida e contas de datas
//
// Guarda as preferências do usuário (intervalo, modo, cor, impressora, datas)
// num arquivo JSON via plugin `store` do Tauri. As datas são guardadas como
// "unix timestamp" (número de segundos desde 01/01/1970), que é fácil de
// comparar e somar.
// ============================================================================

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt; // habilita app.store(...)

const STORE_FILE: &str = "settings.json"; // nome do arquivo de configuração
const CONFIG_KEY: &str = "config"; // chave dentro do JSON
const DAY_SECS: u64 = 86_400; // segundos em um dia (24 * 60 * 60)

/// Configuração persistida do app.
/// `Serialize`/`Deserialize` permitem converter de/para JSON; `#[serde(default)]`
/// faz campos ausentes assumirem o valor padrão (útil ao adicionar campos novos).
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

/// Valores iniciais quando ainda não há nada salvo (primeira execução).
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

/// "Agora" em unix timestamp (segundos).
pub fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Lê a configuração do arquivo. Se não existir ou der erro, devolve o padrão.
pub fn load_config<R: Runtime>(app: &AppHandle<R>) -> Config {
    match app.store(STORE_FILE) {
        Ok(store) => store
            .get(CONFIG_KEY)
            // `from_value` tenta transformar o JSON na struct Config.
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

/// Grava a configuração no arquivo (e salva em disco).
pub fn save_config<R: Runtime>(app: &AppHandle<R>, cfg: &Config) {
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(CONFIG_KEY, json!(cfg)); // json!(cfg) converte a struct em JSON
        let _ = store.save();
    }
}

/// Timestamp em que a próxima impressão fica devida (última + intervalo).
pub fn next_due_ts(cfg: &Config) -> Option<u64> {
    cfg.last_print_ts
        .map(|t| t + cfg.interval_days as u64 * DAY_SECS)
}

/// Dias restantes até a próxima impressão (negativo = atrasado).
/// `div_euclid` divide arredondando "para baixo" mesmo com números negativos.
pub fn days_left(cfg: &Config) -> i64 {
    match next_due_ts(cfg) {
        Some(due) => (due as i64 - now_ts() as i64).div_euclid(DAY_SECS as i64),
        None => 0,
    }
}

/// Já passou do intervalo desde a última impressão? (Sem data = sim.)
pub fn is_due(cfg: &Config) -> bool {
    match cfg.last_print_ts {
        Some(t) => now_ts() >= t + cfg.interval_days as u64 * DAY_SECS,
        None => true,
    }
}
