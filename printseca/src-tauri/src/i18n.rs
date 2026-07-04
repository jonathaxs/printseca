// ============================================================================
// i18n.rs — Internacionalização do "backend" (textos do Rust)
//
// Traduz o que aparece FORA da janela web: o menu da bandeja e as notificações
// do sistema. (Os textos da janela em si são traduzidos no frontend, em
// src/i18n.ts.)
//
// Idiomas: Português ("pt") e Inglês ("en"). A escolha vem da config:
//   • "auto" -> detectamos o idioma do sistema operacional (sys_locale);
//   • "pt" / "en" -> o usuário fixou um idioma no seletor.
// Qualquer idioma de sistema que não comece com "pt" cai no inglês.
// ============================================================================

use crate::settings::Config;

/// Os dois idiomas que o app conhece.
#[derive(Clone, Copy, PartialEq)]
pub enum Lang {
    Pt,
    En,
}

/// Resolve o idioma efetivo a partir da preferência salva ("auto"/"pt"/"en").
pub fn resolve(cfg: &Config) -> Lang {
    let code = match cfg.lang.as_str() {
        "pt" => "pt".to_string(),
        "en" => "en".to_string(),
        // "auto" (ou qualquer valor estranho): pergunta ao sistema operacional.
        _ => sys_locale::get_locale().unwrap_or_default().to_lowercase(),
    };
    if code.starts_with("pt") {
        Lang::Pt
    } else {
        Lang::En
    }
}

/// Código curto ("pt"/"en") — enviado ao frontend para ele escolher o dicionário.
pub fn code(lang: Lang) -> &'static str {
    match lang {
        Lang::Pt => "pt",
        Lang::En => "en",
    }
}

/// Rótulos fixos dos itens do menu da bandeja.
pub struct TrayLabels {
    pub print: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
}

pub fn tray_labels(lang: Lang) -> TrayLabels {
    match lang {
        Lang::Pt => TrayLabels {
            print: "Imprimir agora",
            settings: "Configurações…",
            quit: "Sair",
        },
        Lang::En => TrayLabels {
            print: "Print now",
            settings: "Settings…",
            quit: "Quit",
        },
    }
}

/// Texto de status no topo do menu da bandeja, já com as contas de dias.
/// `has_baseline` = já houve uma primeira impressão registrada.
pub fn status_text(lang: Lang, days_left: i64, has_baseline: bool) -> String {
    match lang {
        Lang::Pt => {
            if !has_baseline {
                "Configurando…".into()
            } else if days_left > 1 {
                format!("Próxima impressão em {days_left} dias")
            } else if days_left == 1 {
                "Próxima impressão amanhã".into()
            } else if days_left == 0 {
                "Imprimir hoje".into()
            } else {
                format!("Atrasado há {} dia(s)", -days_left)
            }
        }
        Lang::En => {
            if !has_baseline {
                "Setting up…".into()
            } else if days_left > 1 {
                format!("Next print in {days_left} days")
            } else if days_left == 1 {
                "Next print tomorrow".into()
            } else if days_left == 0 {
                "Print today".into()
            } else {
                format!("Overdue by {} day(s)", -days_left)
            }
        }
    }
}

// ---- Notificações (título + corpo) ----
// Cada função devolve um par (título, corpo). Onde há detalhe dinâmico (o erro
// da impressora), o lib.rs monta o corpo final combinando com estes textos.

/// Modo automático imprimiu sozinho com sucesso.
pub fn notif_auto_ok(lang: Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::Pt => (
            "Impressão de manutenção concluída",
            "O Printseca imprimiu a página para manter a tinta fluindo.",
        ),
        Lang::En => (
            "Maintenance print done",
            "Printseca printed the page to keep the ink flowing.",
        ),
    }
}

/// Chegou a hora, no modo "avisar".
pub fn notif_due(lang: Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::Pt => (
            "Hora de imprimir!",
            "Abra o Printseca e clique em \"Imprimir agora\" para não deixar a tinta secar.",
        ),
        Lang::En => (
            "Time to print!",
            "Open Printseca and click \"Print now\" so the ink doesn't dry out.",
        ),
    }
}

/// Impressão manual (pela bandeja) enviada com sucesso.
pub fn notif_sent(lang: Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::Pt => (
            "Impressão enviada",
            "Página de manutenção enviada para a impressora.",
        ),
        Lang::En => (
            "Print sent",
            "Maintenance page sent to the printer.",
        ),
    }
}

/// Título usado quando a impressão falha.
pub fn notif_fail_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Pt => "Não consegui imprimir",
        Lang::En => "Couldn't print",
    }
}

/// Prefixo do corpo quando a impressão automática falha (segue o erro entre
/// parênteses, montado no lib.rs).
pub fn notif_check_printer(lang: Lang) -> &'static str {
    match lang {
        Lang::Pt => "Verifique a impressora.",
        Lang::En => "Check the printer.",
    }
}
