import { invoke } from "@tauri-apps/api/core";

interface State {
  interval_days: number;
  mode: string;
  color: boolean;
  printer: string | null;
  last_print_ts: number | null;
  next_due_ts: number | null;
  days_left: number;
  autostart: boolean;
  printers: string[];
}

const $ = <T extends HTMLElement = HTMLElement>(sel: string) =>
  document.querySelector(sel) as T;

let toastTimer: number | undefined;
function toast(msg: string, err = false) {
  const t = $("#toast");
  t.textContent = msg;
  t.classList.toggle("err", err);
  t.classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => t.classList.remove("show"), 2600);
}

function fmtDate(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleDateString("pt-BR", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

function setRadio(name: string, value: string) {
  const el = document.querySelector(
    `input[name="${name}"][value="${value}"]`,
  ) as HTMLInputElement | null;
  if (el) el.checked = true;
}

function highlightChip(v: number) {
  document
    .querySelectorAll<HTMLButtonElement>("#interval-chips button")
    .forEach((b) => b.classList.toggle("active", Number(b.dataset.v) === v));
}

function render(s: State) {
  const card = $("#status-card");
  const daysEl = $("#days");
  const label = $("#days-label");
  card.classList.remove("warn", "late");

  if (s.last_print_ts == null) {
    daysEl.textContent = "—";
    label.textContent = "configurando…";
  } else if (s.days_left < 0) {
    daysEl.textContent = String(Math.abs(s.days_left));
    label.textContent = s.days_left === -1 ? "dia atrasado" : "dias atrasados";
    card.classList.add("late");
  } else if (s.days_left === 0) {
    daysEl.textContent = "0";
    label.textContent = "imprima hoje";
    card.classList.add("warn");
  } else {
    daysEl.textContent = String(s.days_left);
    label.textContent = s.days_left === 1 ? "dia restante" : "dias restantes";
    if (s.days_left <= 2) card.classList.add("warn");
  }

  $("#last-print").textContent = fmtDate(s.last_print_ts);
  $("#next-due").textContent = fmtDate(s.next_due_ts);

  ($("#interval") as HTMLInputElement).value = String(s.interval_days);
  highlightChip(s.interval_days);
  setRadio("mode", s.mode);
  setRadio("color", String(s.color));
  ($("#autostart") as HTMLInputElement).checked = s.autostart;

  const sel = $("#printer") as HTMLSelectElement;
  sel.innerHTML = "";
  sel.add(new Option("Padrão do sistema", ""));
  for (const p of s.printers) sel.add(new Option(p, p));
  sel.value = s.printer ?? "";
}

async function refresh() {
  render(await invoke<State>("get_state"));
}

async function save() {
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
  render(
    await invoke<State>("save_config", {
      intervalDays: interval,
      mode,
      color,
      printer,
    }),
  );
  toast("Configurações salvas");
}

window.addEventListener("DOMContentLoaded", () => {
  $("#print-now").addEventListener("click", async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      render(await invoke<State>("print_now"));
      toast("Página enviada para a impressora");
    } catch (err) {
      toast(String(err), true);
    } finally {
      btn.disabled = false;
    }
  });

  $("#mark-printed").addEventListener("click", async () => {
    render(await invoke<State>("mark_printed"));
    toast("Marcado como impresso");
  });

  $("#save").addEventListener("click", save);

  $("#interval-chips").addEventListener("click", (e) => {
    const b = (e.target as HTMLElement).closest("button");
    if (!b) return;
    ($("#interval") as HTMLInputElement).value = b.dataset.v!;
    highlightChip(Number(b.dataset.v));
  });

  ($("#interval") as HTMLInputElement).addEventListener("input", (e) =>
    highlightChip(Number((e.target as HTMLInputElement).value)),
  );

  ($("#autostart") as HTMLInputElement).addEventListener("change", async (e) => {
    const cb = e.target as HTMLInputElement;
    try {
      cb.checked = await invoke<boolean>("set_autostart", {
        enabled: cb.checked,
      });
      toast(
        cb.checked
          ? "Inicialização automática ativada"
          : "Inicialização automática desativada",
      );
    } catch (err) {
      cb.checked = !cb.checked;
      toast(String(err), true);
    }
  });

  refresh();
});
