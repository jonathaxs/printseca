// ============================================================================
// i18n.ts — Internacionalização da JANELA (frontend)
//
// Um "dicionário" simples: para cada idioma, um mapa de chave -> texto. O idioma
// efetivo ("pt" ou "en") vem do backend (Rust) dentro do estado, que já sabe
// seguir o sistema operacional. Aqui só escolhemos as strings certas.
//
// Como usar:
//   • t(lang, "chave")     -> devolve o texto traduzido (para strings dinâmicas).
//   • applyStaticI18n(lang) -> percorre o HTML e traduz todo elemento que tenha
//                              o atributo data-i18n="chave".
// ============================================================================

export type Lang = "pt" | "en";

type Dict = Record<string, string>;

const pt: Dict = {
  // --- textos fixos do HTML (data-i18n) ---
  "app.subtitle": "Mantém a tinta da impressora fluindo",
  "status.loading": "carregando…",
  "status.lastPrint": "Última impressão",
  "status.next": "Próxima",
  "action.printNow": "Imprimir agora",
  "action.markPrinted": "Já imprimi manualmente",
  "settings.title": "Configurações",
  "settings.interval": "Imprimir a cada",
  "settings.days": "dias",
  "settings.whenDue": "Quando vencer",
  "mode.notify.title": "Avisar",
  "mode.notify.desc": "Notifica e você clica em imprimir",
  "mode.auto.title": "Automático",
  "mode.auto.desc": "Imprime sozinho se a impressora estiver pronta",
  "settings.page": "Página",
  "color.true.title": "Colorida",
  "color.true.desc": "Exercita todas as tintas (CMYK)",
  "color.false.title": "Preto e branco",
  "color.false.desc": "Usa só a tinta preta",
  "settings.printer": "Impressora",
  "settings.printerDefault": "Padrão do sistema",
  "settings.appearance": "Aparência",
  "theme.auto": "Automático (sistema)",
  "theme.light": "Claro",
  "theme.dark": "Escuro",
  "settings.autostart": "Iniciar junto com o sistema",
  "settings.language": "Idioma",
  "lang.auto": "Automático (sistema)",
  "settings.save": "Salvar configurações",

  // --- strings dinâmicas (usadas no main.ts) ---
  "days.settingUp": "configurando…",
  "days.today": "imprima hoje",
  "days.leftOne": "dia restante",
  "days.leftMany": "dias restantes",
  "days.lateOne": "dia atrasado",
  "days.lateMany": "dias atrasados",
  "toast.saved": "Configurações salvas",
  "toast.sent": "Página enviada para a impressora",
  "toast.marked": "Marcado como impresso",
  "toast.autostartOn": "Inicialização automática ativada",
  "toast.autostartOff": "Inicialização automática desativada",
};

const en: Dict = {
  "app.subtitle": "Keeps your printer's ink flowing",
  "status.loading": "loading…",
  "status.lastPrint": "Last print",
  "status.next": "Next",
  "action.printNow": "Print now",
  "action.markPrinted": "I already printed manually",
  "settings.title": "Settings",
  "settings.interval": "Print every",
  "settings.days": "days",
  "settings.whenDue": "When due",
  "mode.notify.title": "Notify",
  "mode.notify.desc": "Notifies you and you click print",
  "mode.auto.title": "Automatic",
  "mode.auto.desc": "Prints on its own if the printer is ready",
  "settings.page": "Page",
  "color.true.title": "Color",
  "color.true.desc": "Exercises every ink (CMYK)",
  "color.false.title": "Black and white",
  "color.false.desc": "Uses only the black ink",
  "settings.printer": "Printer",
  "settings.printerDefault": "System default",
  "settings.appearance": "Appearance",
  "theme.auto": "Automatic (system)",
  "theme.light": "Light",
  "theme.dark": "Dark",
  "settings.autostart": "Start with the system",
  "settings.language": "Language",
  "lang.auto": "Automatic (system)",
  "settings.save": "Save settings",

  "days.settingUp": "setting up…",
  "days.today": "print today",
  "days.leftOne": "day left",
  "days.leftMany": "days left",
  "days.lateOne": "day overdue",
  "days.lateMany": "days overdue",
  "toast.saved": "Settings saved",
  "toast.sent": "Page sent to the printer",
  "toast.marked": "Marked as printed",
  "toast.autostartOn": "Start at login enabled",
  "toast.autostartOff": "Start at login disabled",
};

const dicts: Record<Lang, Dict> = { pt, en };

// Devolve o texto traduzido. Se faltar a chave, cai no português e, por fim, na
// própria chave (assim nunca fica em branco e fica fácil achar o que faltou).
export function t(lang: Lang, key: string): string {
  return dicts[lang]?.[key] ?? dicts.pt[key] ?? key;
}

// Locale para formatar datas conforme o idioma.
export function dateLocale(lang: Lang): string {
  return lang === "pt" ? "pt-BR" : "en-US";
}

// Traduz todos os elementos do HTML marcados com data-i18n="chave".
export function applyStaticI18n(lang: Lang): void {
  document.querySelectorAll<HTMLElement>("[data-i18n]").forEach((el) => {
    const key = el.dataset.i18n;
    if (key) el.textContent = t(lang, key);
  });
}
