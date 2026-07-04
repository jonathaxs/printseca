// ============================================================================
// main.ts — Lógica da janela de configuração do Printseca (frontend)
//
// Este arquivo roda DENTRO do webview (a "página web" que o Tauri exibe numa
// janela nativa). Ele NÃO acessa impressora, disco, etc. diretamente. Para
// qualquer coisa "de sistema", ele chama funções escritas em Rust
// (ver src-tauri/src/lib.rs) através de `invoke("nome_do_comando", { args })`.
//
// Fluxo geral:
//   1. Ao abrir a janela, pedimos o estado atual ao Rust (get_state).
//   2. Desenhamos esse estado na tela (render) — inclusive traduzindo os textos.
//   3. Quando o usuário clica em algo, chamamos o comando Rust correspondente
//      (print_now, save_config, ...), que devolve o estado já atualizado.
// ============================================================================

import { invoke } from "@tauri-apps/api/core";
import { applyStaticI18n, dateLocale, t, type Lang } from "./i18n";

// Espelho, em TypeScript, da struct `StateView` do Rust (lib.rs). Os nomes dos
// campos precisam bater com o que o Rust serializa (em snake_case).
interface State {
  interval_days: number;
  mode: string; // "notify" (avisa) ou "auto" (imprime sozinho)
  color: boolean; // true = página colorida, false = preto e branco
  printer: string | null; // null = impressora padrão do sistema
  last_print_ts: number | null; // unix em segundos
  next_due_ts: number | null;
  days_left: number; // negativo = atrasado
  autostart: boolean;
  printers: string[];
  lang: Lang; // idioma EFETIVO já resolvido pelo Rust ("pt"/"en")
  lang_pref: string; // preferência salva ("auto"/"pt"/"en")
}

// Idioma efetivo atual. É atualizado a cada render() e usado pelas mensagens
// dinâmicas (toasts, rótulo de dias, datas). Começa em "pt" só como fallback.
let lang: Lang = "pt";

// Atalho para document.querySelector já com o tipo certo — só para digitar menos.
const $ = <T extends HTMLElement = HTMLElement>(sel: string) =>
  document.querySelector(sel) as T;

// Mostra uma mensagem rápida (toast) no rodapé. `err = true` deixa vermelha.
let toastTimer: number | undefined;
function toast(msg: string, err = false) {
  const el = $("#toast");
  el.textContent = msg;
  el.classList.toggle("err", err);
  el.classList.add("show");
  clearTimeout(toastTimer); // reinicia o cronômetro se vier outro toast antes
  toastTimer = window.setTimeout(() => el.classList.remove("show"), 2600);
}

// Converte um timestamp unix (segundos) em data legível no idioma atual.
// O Rust manda em segundos; o JavaScript trabalha em milissegundos -> * 1000.
function fmtDate(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleDateString(dateLocale(lang), {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

// Marca o radio button (do grupo `name`) cujo valor é `value`.
function setRadio(name: string, value: string) {
  const el = document.querySelector(
    `input[name="${name}"][value="${value}"]`,
  ) as HTMLInputElement | null;
  if (el) el.checked = true;
}

// Destaca o "chip" de atalho (7/10/14/20) que corresponde ao intervalo atual.
function highlightChip(v: number) {
  document
    .querySelectorAll<HTMLButtonElement>("#interval-chips button")
    .forEach((b) => b.classList.toggle("active", Number(b.dataset.v) === v));
}

// Recebe um estado vindo do Rust e atualiza TODA a tela com ele. É a única
// função que "desenha": sempre que algo muda, chamamos render() de novo.
function render(s: State) {
  // 1) Idioma: guarda o efetivo e traduz todo o texto fixo do HTML (data-i18n).
  lang = s.lang;
  applyStaticI18n(lang);

  const card = $("#status-card");
  const daysEl = $("#days");
  const label = $("#days-label");
  card.classList.remove("warn", "late"); // limpa as cores antes de recalcular

  // Cartão de status: número grande + texto, com cor conforme a urgência.
  if (s.last_print_ts == null) {
    daysEl.textContent = "—";
    label.textContent = t(lang, "days.settingUp");
  } else if (s.days_left < 0) {
    daysEl.textContent = String(Math.abs(s.days_left));
    label.textContent =
      s.days_left === -1 ? t(lang, "days.lateOne") : t(lang, "days.lateMany");
    card.classList.add("late"); // borda vermelha
  } else if (s.days_left === 0) {
    daysEl.textContent = "0";
    label.textContent = t(lang, "days.today");
    card.classList.add("warn"); // borda amarela
  } else {
    daysEl.textContent = String(s.days_left);
    label.textContent =
      s.days_left === 1 ? t(lang, "days.leftOne") : t(lang, "days.leftMany");
    if (s.days_left <= 2) card.classList.add("warn"); // amarela perto do prazo
  }

  $("#last-print").textContent = fmtDate(s.last_print_ts);
  $("#next-due").textContent = fmtDate(s.next_due_ts);

  // Preenche o formulário com os valores salvos.
  ($("#interval") as HTMLInputElement).value = String(s.interval_days);
  highlightChip(s.interval_days);
  setRadio("mode", s.mode);
  setRadio("color", String(s.color));
  ($("#lang") as HTMLSelectElement).value = s.lang_pref;
  ($("#autostart") as HTMLInputElement).checked = s.autostart;

  // Monta a lista de impressoras: "Padrão do sistema" + as detectadas pelo Rust.
  const sel = $("#printer") as HTMLSelectElement;
  sel.innerHTML = "";
  sel.add(new Option(t(lang, "settings.printerDefault"), "")); // value "" = padrão
  for (const p of s.printers) sel.add(new Option(p, p));
  sel.value = s.printer ?? "";
}

// Pede o estado atual ao Rust e redesenha. `invoke<State>` avisa ao TypeScript
// que esse comando devolve um objeto do tipo State.
async function refresh() {
  render(await invoke<State>("get_state"));
}

// Lê o formulário e manda salvar no Rust, que devolve o estado já atualizado.
async function save() {
  // Garante um número entre 1 e 365 (e 10 como padrão se o campo estiver vazio).
  const interval = Math.max(
    1,
    Math.min(365, Number(($("#interval") as HTMLInputElement).value) || 10),
  );
  const mode =
    (document.querySelector('input[name="mode"]:checked') as HTMLInputElement)
      ?.value ?? "notify";
  const color =
    (document.querySelector('input[name="color"]:checked') as HTMLInputElement)
      ?.value === "true";
  const printer = ($("#printer") as HTMLSelectElement).value || null;
  const langPref = ($("#lang") as HTMLSelectElement).value;

  // OBS: o Tauri converte camelCase (JS) -> snake_case (Rust) automaticamente,
  // por isso enviamos `intervalDays` e o Rust recebe `interval_days`.
  const st = await invoke<State>("save_config", {
    intervalDays: interval,
    mode,
    color,
    printer,
    lang: langPref,
  });
  render(st); // pode vir noutro idioma (se o usuário trocou o seletor)
  toast(t(st.lang, "toast.saved"));
}

// Tudo abaixo só roda depois que o HTML terminou de carregar.
window.addEventListener("DOMContentLoaded", () => {
  // "Imprimir agora": desabilita o botão enquanto imprime (evita clique duplo).
  $("#print-now").addEventListener("click", async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      render(await invoke<State>("print_now"));
      toast(t(lang, "toast.sent"));
    } catch (err) {
      // Se o Rust devolver Err(...), o invoke "lança" (throw) e caímos aqui.
      toast(String(err), true);
    } finally {
      btn.disabled = false;
    }
  });

  // "Já imprimi manualmente": só zera o contador, sem mandar imprimir nada.
  $("#mark-printed").addEventListener("click", async () => {
    render(await invoke<State>("mark_printed"));
    toast(t(lang, "toast.marked"));
  });

  $("#save").addEventListener("click", save);

  // Clicar num chip (7/10/14/20) preenche o campo de intervalo.
  $("#interval-chips").addEventListener("click", (e) => {
    const b = (e.target as HTMLElement).closest("button");
    if (!b) return;
    ($("#interval") as HTMLInputElement).value = b.dataset.v!;
    highlightChip(Number(b.dataset.v));
  });

  // Digitar no campo de intervalo atualiza o destaque dos chips em tempo real.
  ($("#interval") as HTMLInputElement).addEventListener("input", (e) =>
    highlightChip(Number((e.target as HTMLInputElement).value)),
  );

  // O toggle de autostart aplica na hora (não espera o "Salvar"). Se falhar,
  // revertemos o estado visual do checkbox.
  ($("#autostart") as HTMLInputElement).addEventListener("change", async (e) => {
    const cb = e.target as HTMLInputElement;
    try {
      cb.checked = await invoke<boolean>("set_autostart", {
        enabled: cb.checked,
      });
      toast(t(lang, cb.checked ? "toast.autostartOn" : "toast.autostartOff"));
    } catch (err) {
      cb.checked = !cb.checked; // reverte se deu erro
      toast(String(err), true);
    }
  });

  refresh(); // primeira carga: pega o estado e desenha a tela
});
