mod printing;
mod settings;

use std::time::Duration;

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Runtime, Wry,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

use settings::Config;

/// Handles guardados em estado para atualizar a bandeja a partir do agendador.
struct TrayHandles {
    status_item: MenuItem<Wry>,
}

/// Visão de estado enviada ao frontend.
#[derive(Serialize)]
struct StateView {
    interval_days: u32,
    mode: String,
    color: bool,
    printer: Option<String>,
    last_print_ts: Option<u64>,
    next_due_ts: Option<u64>,
    days_left: i64,
    autostart: bool,
    printers: Vec<String>,
}

fn build_state_view<R: Runtime>(app: &AppHandle<R>, cfg: &Config) -> StateView {
    StateView {
        interval_days: cfg.interval_days,
        mode: cfg.mode.clone(),
        color: cfg.color,
        printer: cfg.printer.clone(),
        last_print_ts: cfg.last_print_ts,
        next_due_ts: settings::next_due_ts(cfg),
        days_left: settings::days_left(cfg),
        autostart: app.autolaunch().is_enabled().unwrap_or(false),
        printers: printing::list_printers(),
    }
}

fn status_text(cfg: &Config) -> String {
    match cfg.last_print_ts {
        None => "Configurando…".into(),
        Some(_) => {
            let d = settings::days_left(cfg);
            if d > 1 {
                format!("Próxima impressão em {d} dias")
            } else if d == 1 {
                "Próxima impressão amanhã".into()
            } else if d == 0 {
                "Imprimir hoje".into()
            } else {
                format!("Atrasado há {} dia(s)", -d)
            }
        }
    }
}

fn update_tray_status<R: Runtime>(app: &AppHandle<R>, cfg: &Config) {
    if let Some(state) = app.try_state::<TrayHandles>() {
        let _ = state.status_item.set_text(status_text(cfg));
    }
}

fn show_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn notify(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

/// Throttle: no máximo ~uma notificação a cada 20h.
fn should_notify(cfg: &Config) -> bool {
    match cfg.last_notified_ts {
        None => true,
        Some(t) => settings::now_ts() >= t + 20 * 3600,
    }
}

/// Executa uma impressão e atualiza o estado em caso de sucesso.
fn do_print(app: &AppHandle, cfg: &mut Config) -> Result<(), String> {
    printing::print_pdf(app, cfg.color, cfg.printer.as_deref())?;
    cfg.last_print_ts = Some(settings::now_ts());
    settings::save_config(app, cfg);
    update_tray_status(app, cfg);
    Ok(())
}

/// Verificação periódica do agendador.
fn check_schedule(app: &AppHandle) {
    let mut cfg = settings::load_config(app);

    // Primeira execução: estabelece a linha de base (conta a partir de agora).
    if cfg.last_print_ts.is_none() {
        cfg.last_print_ts = Some(settings::now_ts());
        settings::save_config(app, &cfg);
        update_tray_status(app, &cfg);
        return;
    }

    update_tray_status(app, &cfg);

    if !settings::is_due(&cfg) {
        return;
    }

    if cfg.mode == "auto" {
        match do_print(app, &mut cfg) {
            Ok(()) => notify(
                app,
                "Impressão de manutenção concluída",
                "O Printseca imprimiu a página para manter a tinta fluindo.",
            ),
            Err(e) => {
                if should_notify(&cfg) {
                    notify(
                        app,
                        "Não consegui imprimir",
                        &format!("Verifique a impressora. ({e})"),
                    );
                    cfg.last_notified_ts = Some(settings::now_ts());
                    settings::save_config(app, &cfg);
                }
            }
        }
    } else if should_notify(&cfg) {
        notify(
            app,
            "Hora de imprimir!",
            "Abra o Printseca e clique em \"Imprimir agora\" para não deixar a tinta secar.",
        );
        cfg.last_notified_ts = Some(settings::now_ts());
        settings::save_config(app, &cfg);
    }
}

// ---------- Comandos chamados pelo frontend ----------

#[tauri::command]
fn get_state(app: AppHandle) -> StateView {
    let cfg = settings::load_config(&app);
    build_state_view(&app, &cfg)
}

#[tauri::command]
fn save_config(
    app: AppHandle,
    interval_days: u32,
    mode: String,
    color: bool,
    printer: Option<String>,
) -> StateView {
    let mut cfg = settings::load_config(&app);
    cfg.interval_days = interval_days.clamp(1, 365);
    cfg.mode = if mode == "auto" { "auto" } else { "notify" }.into();
    cfg.color = color;
    cfg.printer = printer.filter(|s| !s.is_empty());
    settings::save_config(&app, &cfg);
    update_tray_status(&app, &cfg);
    build_state_view(&app, &cfg)
}

#[tauri::command]
fn print_now(app: AppHandle) -> Result<StateView, String> {
    let mut cfg = settings::load_config(&app);
    do_print(&app, &mut cfg)?;
    Ok(build_state_view(&app, &cfg))
}

#[tauri::command]
fn mark_printed(app: AppHandle) -> StateView {
    let mut cfg = settings::load_config(&app);
    cfg.last_print_ts = Some(settings::now_ts());
    settings::save_config(&app, &cfg);
    update_tray_status(&app, &cfg);
    build_state_view(&app, &cfg)
}

#[tauri::command]
fn list_printers() -> Vec<String> {
    printing::list_printers()
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(|e| e.to_string())?;
    } else {
        mgr.disable().map_err(|e| e.to_string())?;
    }
    mgr.is_enabled().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_settings_window(app);
        }))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_config,
            print_now,
            mark_printed,
            list_printers,
            set_autostart
        ])
        .setup(|app| {
            // macOS: app de barra de menu, sem ícone no Dock.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let cfg = settings::load_config(&app.handle().clone());

            // --- Bandeja (system tray) ---
            let status_item =
                MenuItem::with_id(app, "status", status_text(&cfg), false, None::<&str>)?;
            let print_item =
                MenuItem::with_id(app, "print", "Imprimir agora", true, None::<&str>)?;
            let settings_item =
                MenuItem::with_id(app, "settings", "Configurações…", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &status_item,
                    &PredefinedMenuItem::separator(app)?,
                    &print_item,
                    &settings_item,
                    &PredefinedMenuItem::separator(app)?,
                    &quit_item,
                ],
            )?;

            let _tray = TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Printseca")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "print" => {
                        let app = app.clone();
                        std::thread::spawn(move || {
                            let mut cfg = settings::load_config(&app);
                            match do_print(&app, &mut cfg) {
                                Ok(()) => notify(
                                    &app,
                                    "Impressão enviada",
                                    "Página de manutenção enviada para a impressora.",
                                ),
                                Err(e) => notify(&app, "Não consegui imprimir", &e),
                            }
                        });
                    }
                    "settings" => show_settings_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            app.manage(TrayHandles { status_item });

            // --- Agendador ---
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(5));
                loop {
                    check_schedule(&handle);
                    std::thread::sleep(Duration::from_secs(30 * 60));
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Fechar a janela apenas esconde — o app continua na bandeja.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
