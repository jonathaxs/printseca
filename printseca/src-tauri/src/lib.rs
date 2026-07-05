// ============================================================================
// lib.rs — "Backend" do Printseca (código Rust)
//
// Enquanto o frontend (src/main.ts) cuida da aparência, este arquivo cuida de
// tudo que mexe com o sistema: bandeja, notificações, agendamento e impressão.
//
// Conceitos do Tauri que aparecem aqui:
//   • #[tauri::command] -> marca uma função Rust que pode ser chamada do JS
//                          via invoke("nome", { args }).
//   • tauri::Builder    -> "monta" o app: registra plugins, comandos e o setup.
//   • plugins           -> store (salvar config), notification (avisos),
//                          autostart (iniciar com o sistema) e single-instance
//                          (impedir abrir duas vezes).
//   • AppHandle         -> uma "alça" para o app; serve para acessar janelas,
//                          plugins e estado a partir de qualquer lugar.
// ============================================================================

mod i18n; // traduções do backend (menu da bandeja e notificações)
mod printing; // funções de impressão (lp no mac/linux, SumatraPDF no Windows)
mod settings; // carregar/salvar a configuração e contas de datas

use std::time::Duration;

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Runtime, Wry,
};
use tauri_plugin_autostart::ManagerExt; // habilita app.autolaunch()
use tauri_plugin_notification::NotificationExt; // habilita app.notification()

use settings::Config;

/// Guardamos as "alças" dos itens do menu para mudar os textos depois: o status
/// (ex.: "Próxima impressão em 5 dias") a partir do agendador, e os demais
/// rótulos quando o usuário troca o idioma no seletor.
struct TrayHandles {
    status_item: MenuItem<Wry>,
    print_item: MenuItem<Wry>,
    settings_item: MenuItem<Wry>,
    quit_item: MenuItem<Wry>,
}

/// Pacote de estado que enviamos ao frontend. O `#[derive(Serialize)]` ensina o
/// Rust a transformar isso em JSON automaticamente; os nomes dos campos viram
/// as chaves que o main.ts lê (a interface `State`).
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
    /// Idioma EFETIVO já resolvido ("pt"/"en") — o frontend usa para traduzir.
    lang: String,
    /// Preferência salva ("auto"/"pt"/"en") — o que o seletor mostra selecionado.
    lang_pref: String,
    /// Aparência salva ("auto"/"light"/"dark") — aplicada via data-theme na janela.
    theme: String,
}

// --- Ícone da bandeja por plataforma ---
// macOS: ícone PRETO marcado como "template"; o sistema tinge sozinho (preto no
//        modo claro, branco no escuro) — fica perfeito, então mantemos.
// Windows: trocamos entre PRETO (barra clara) e BRANCO (barra escura) conforme o
//        tema, senão o preto some na barra escura do Windows 11.
// Linux: um CINZA fixo, porque detectar o tema do painel varia demais entre
//        GNOME/KDE/etc. — o cinza é legível tanto em barra clara quanto escura.
#[cfg(target_os = "macos")]
fn tray_icon_bytes(_dark: bool) -> &'static [u8] {
    include_bytes!("../icons/tray.png")
}
#[cfg(target_os = "linux")]
fn tray_icon_bytes(_dark: bool) -> &'static [u8] {
    include_bytes!("../icons/tray-gray.png")
}
#[cfg(target_os = "windows")]
fn tray_icon_bytes(dark: bool) -> &'static [u8] {
    if dark {
        include_bytes!("../icons/tray-white.png")
    } else {
        include_bytes!("../icons/tray.png")
    }
}

/// Monta a imagem do ícone da bandeja. `dark` = a barra/tema está no modo escuro
/// (só influencia no Windows; nos demais o parâmetro é ignorado).
fn tray_image(dark: bool) -> tauri::image::Image<'static> {
    tauri::image::Image::from_bytes(tray_icon_bytes(dark)).expect("ícone da bandeja inválido")
}

/// Monta o StateView juntando a config salva + dados "ao vivo" (lista de
/// impressoras e se o autostart está ligado).
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
        lang: i18n::code(i18n::resolve(cfg)).into(),
        lang_pref: cfg.lang.clone(),
        theme: cfg.theme.clone(),
    }
}

/// Texto curto que aparece no topo do menu da bandeja (já traduzido).
fn status_text(cfg: &Config) -> String {
    i18n::status_text(
        i18n::resolve(cfg),
        settings::days_left(cfg),
        cfg.last_print_ts.is_some(),
    )
}

/// Atualiza o texto do item de status no menu (se a bandeja já existir).
/// `try_state` recupera o TrayHandles que guardamos com `app.manage(...)`.
fn update_tray_status<R: Runtime>(app: &AppHandle<R>, cfg: &Config) {
    if let Some(state) = app.try_state::<TrayHandles>() {
        let _ = state.status_item.set_text(status_text(cfg));
    }
}

/// Retraduz TODOS os itens do menu da bandeja (usado ao trocar o idioma).
fn update_tray_lang<R: Runtime>(app: &AppHandle<R>, cfg: &Config) {
    if let Some(state) = app.try_state::<TrayHandles>() {
        let labels = i18n::tray_labels(i18n::resolve(cfg));
        let _ = state.status_item.set_text(status_text(cfg));
        let _ = state.print_item.set_text(labels.print);
        let _ = state.settings_item.set_text(labels.settings);
        let _ = state.quit_item.set_text(labels.quit);
    }
}

/// Mostra (e foca) a janela de configuração. Lembre: ela nasce escondida.
fn show_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

/// Atalho para disparar uma notificação do sistema.
fn notify(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

/// Anti-spam: deixa notificar no máximo ~uma vez a cada 20h.
fn should_notify(cfg: &Config) -> bool {
    match cfg.last_notified_ts {
        None => true,
        Some(t) => settings::now_ts() >= t + 20 * 3600,
    }
}

/// Imprime e, se der certo, registra "agora" como a última impressão.
/// Recebe `&mut Config` porque altera a data dentro dele.
fn do_print(app: &AppHandle, cfg: &mut Config) -> Result<(), String> {
    printing::print_pdf(app, cfg.color, cfg.printer.as_deref())?;
    cfg.last_print_ts = Some(settings::now_ts());
    settings::save_config(app, cfg);
    update_tray_status(app, cfg);
    Ok(())
}

/// O coração do agendador: roda de tempos em tempos e decide o que fazer.
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

    // Ainda não venceu o intervalo? Não faz nada.
    if !settings::is_due(&cfg) {
        return;
    }

    // Idioma para as notificações (não muda durante esta função).
    let lang = i18n::resolve(&cfg);

    if cfg.mode == "auto" {
        // Modo automático: tenta imprimir sozinho.
        match do_print(app, &mut cfg) {
            Ok(()) => {
                let (title, body) = i18n::notif_auto_ok(lang);
                notify(app, title, body);
            }
            Err(e) => {
                // Falhou (impressora desligada/sem papel): avisa, sem insistir.
                if should_notify(&cfg) {
                    let body = format!("{} ({e})", i18n::notif_check_printer(lang));
                    notify(app, i18n::notif_fail_title(lang), &body);
                    cfg.last_notified_ts = Some(settings::now_ts());
                    settings::save_config(app, &cfg);
                }
            }
        }
    } else if should_notify(&cfg) {
        // Modo "avisar": só notifica e deixa o usuário clicar em Imprimir.
        let (title, body) = i18n::notif_due(lang);
        notify(app, title, body);
        cfg.last_notified_ts = Some(settings::now_ts());
        settings::save_config(app, &cfg);
    }
}

// ---------- Comandos chamados pelo frontend (via invoke) ----------
// Cada função abaixo recebe `app: AppHandle` "de graça" (o Tauri injeta) e pode
// receber argumentos vindos do JS. O que ela retornar vira a resposta da Promise
// do invoke() lá no main.ts. Retornar `Result` permite devolver erro (vira throw).

/// Lê o estado atual e devolve para a tela.
#[tauri::command]
fn get_state(app: AppHandle) -> StateView {
    let cfg = settings::load_config(&app);
    build_state_view(&app, &cfg)
}

/// Salva o formulário. Faz validações: intervalo entre 1 e 365, normaliza o
/// modo e trata impressora vazia como "padrão do sistema" (None).
#[tauri::command]
fn save_config(
    app: AppHandle,
    interval_days: u32,
    mode: String,
    color: bool,
    printer: Option<String>,
    lang: String,
    theme: String,
) -> StateView {
    let mut cfg = settings::load_config(&app);
    cfg.interval_days = interval_days.clamp(1, 365);
    cfg.mode = if mode == "auto" { "auto" } else { "notify" }.into();
    cfg.color = color;
    cfg.printer = printer.filter(|s| !s.is_empty());
    // Só aceita valores conhecidos; qualquer outra coisa vira "auto".
    cfg.lang = match lang.as_str() {
        "pt" | "en" => lang,
        _ => "auto".into(),
    };
    cfg.theme = match theme.as_str() {
        "light" | "dark" => theme,
        _ => "auto".into(),
    };
    settings::save_config(&app, &cfg);
    // Retraduz o menu da bandeja (o idioma pode ter mudado).
    update_tray_lang(&app, &cfg);
    build_state_view(&app, &cfg)
}

/// Imprime agora (botão da janela). Propaga o erro com `?` para o frontend.
#[tauri::command]
fn print_now(app: AppHandle) -> Result<StateView, String> {
    let mut cfg = settings::load_config(&app);
    do_print(&app, &mut cfg)?;
    Ok(build_state_view(&app, &cfg))
}

/// "Já imprimi manualmente": só reinicia o contador, sem imprimir.
#[tauri::command]
fn mark_printed(app: AppHandle) -> StateView {
    let mut cfg = settings::load_config(&app);
    cfg.last_print_ts = Some(settings::now_ts());
    settings::save_config(&app, &cfg);
    update_tray_status(&app, &cfg);
    build_state_view(&app, &cfg)
}

/// Devolve a lista de impressoras do sistema.
#[tauri::command]
fn list_printers() -> Vec<String> {
    printing::list_printers()
}

/// Liga/desliga o "iniciar com o sistema" e devolve o estado real resultante.
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
    // Linux/Wayland: as decorações do GTK bugam no GNOME/KDE em Wayland e os
    // botões da janela (minimizar/maximizar/fechar) ficam sem clicar (bug
    // conhecido do Tauri/tao). Rodar sob XWayland via GDK_BACKEND=x11 contorna
    // isso. Setamos ANTES de o GTK inicializar e só se o usuário não tiver
    // definido nada — assim ele ainda pode forçar Wayland puro se quiser.
    #[cfg(target_os = "linux")]
    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    // O Builder vai "encaixando" peças com .plugin(), .invoke_handler() e
    // .setup(); no fim, .run() inicia o loop principal do app.
    tauri::Builder::default()
        // single-instance precisa ser o PRIMEIRO plugin. Se o app for aberto de
        // novo, em vez de uma 2ª janela ele apenas mostra a já existente.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_settings_window(app);
        }))
        .plugin(tauri_plugin_store::Builder::default().build()) // salva config em disco
        .plugin(tauri_plugin_notification::init()) // notificações do SO
        .plugin(tauri_plugin_autostart::init(
            // iniciar com o sistema
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // Lista branca dos comandos que o frontend pode chamar via invoke(...).
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_config,
            print_now,
            mark_printed,
            list_printers,
            set_autostart
        ])
        // setup() roda uma única vez, na inicialização, com o app já criado.
        .setup(|app| {
            // macOS: vira "app de barra de menu", sem ícone no Dock.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let cfg = settings::load_config(&app.handle().clone());

            // --- Bandeja (system tray) ---
            // Rótulos já no idioma resolvido (segue o sistema ou o que o usuário
            // fixou). MenuItem::with_id(app, id, texto, habilitado?, atalho). O
            // item "status" fica desabilitado (false) de propósito: é só rótulo.
            let labels = i18n::tray_labels(i18n::resolve(&cfg));
            let status_item =
                MenuItem::with_id(app, "status", status_text(&cfg), false, None::<&str>)?;
            let print_item =
                MenuItem::with_id(app, "print", labels.print, true, None::<&str>)?;
            let settings_item =
                MenuItem::with_id(app, "settings", labels.settings, true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;

            // Junta os itens num menu, com separadores entre os grupos.
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

            // Ícone da bandeja (PNG embutido no binário via include_bytes!).
            // Detectamos o tema da janela principal para, no Windows, escolher a
            // versão preta ou branca do ícone (ver tray_icon_bytes acima).
            let dark = app
                .get_webview_window("main")
                .and_then(|w| w.theme().ok())
                .map(|t| t == tauri::Theme::Dark)
                .unwrap_or(false);
            let tray_icon = tray_image(dark);

            // Cria o ícone da bandeja com o menu acima.
            // on_menu_event = o que fazer quando clicam em cada item do menu.
            let tray_builder = TrayIconBuilder::with_id("tray")
                .icon(tray_icon)
                .tooltip("Printseca")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "print" => {
                        // Imprimir pode demorar; rodamos numa thread separada
                        // para não travar a interface enquanto isso acontece.
                        let app = app.clone();
                        std::thread::spawn(move || {
                            let mut cfg = settings::load_config(&app);
                            let lang = i18n::resolve(&cfg);
                            match do_print(&app, &mut cfg) {
                                Ok(()) => {
                                    let (title, body) = i18n::notif_sent(lang);
                                    notify(&app, title, body);
                                }
                                Err(e) => notify(&app, i18n::notif_fail_title(lang), &e),
                            }
                        });
                    }
                    "settings" => show_settings_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                });

            // Só o macOS trata o ícone como "template" (tinge sozinho). No
            // Windows/Linux usamos o PNG como está (preto/branco/cinza).
            #[cfg(target_os = "macos")]
            let tray_builder = tray_builder.icon_as_template(true);

            let _tray = tray_builder.build(app)?;

            // Guarda as alças dos itens no "estado" do app para atualizá-los
            // depois (status a cada ciclo; rótulos ao trocar de idioma).
            app.manage(TrayHandles {
                status_item,
                print_item,
                settings_item,
                quit_item,
            });

            // --- Agendador ---
            // Thread em segundo plano: 5s após abrir e depois a cada 30 min,
            // verifica se está na hora de imprimir/avisar.
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
        // Interceptamos o "fechar janela": em vez de encerrar o app, só a
        // escondemos — ele continua vivo na bandeja.
        .on_window_event(|window, event| match event {
            // Fechar a janela não encerra o app — só esconde (segue na bandeja).
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            // Windows: ao alternar claro/escuro, repinta o ícone da bandeja na
            // hora (preto na barra clara, branco na escura).
            #[cfg(target_os = "windows")]
            tauri::WindowEvent::ThemeChanged(theme) => {
                if let Some(tray) = window.app_handle().tray_by_id("tray") {
                    let dark = *theme == tauri::Theme::Dark;
                    let _ = tray.set_icon(Some(tray_image(dark)));
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
