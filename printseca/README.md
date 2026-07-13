# Printseca

[Português](#português) · [English](#english)

---

## Português

App de desktop que **lembra (e ajuda) a imprimir** uma página de manutenção a cada N
dias, para evitar que a tinta da impressora resseque/entupa por desuso — problema
comum e caro em impressoras de cartucho e de tanque de tinta.

Fica na **bandeja do sistema** (system tray), inicia junto com o sistema e, no
vencimento do intervalo, **notifica** ou **imprime sozinho** uma página que exercita
todas as tintas (ciano, magenta, amarelo e preto) ou apenas preto.

### Status

🚧 Em desenvolvimento inicial.

### Stack

- **[Tauri v2](https://v2.tauri.app/)** (Rust + frontend web)
- Frontend: TypeScript + Vite (vanilla)
- Impressão: CUPS (`lp`) no macOS/Linux; sidecar **SumatraPDF** no Windows

### Plataformas-alvo

| SO       | Formato                             | Arquitetura     |
| -------- | ----------------------------------- | --------------- |
| macOS    | `.zip` com o app pronto (sem assinatura — guia de abertura dentro do zip) | Apple Silicon   |
| Windows  | `.exe` (NSIS)                       | x64             |
| Linux    | `.AppImage` / `.deb` / `.rpm`       | x64             |

### Download

Os pacotes são gerados automaticamente pelo GitHub Actions e ficam nos
**Releases** do repositório. Baixe o formato da sua plataforma:

| Plataforma        | Arquivo                       |
| ----------------- | ----------------------------- |
| macOS             | `Printseca-x.y.z-macos.zip`   |
| Windows           | `Printseca_x.y.z_x64-setup.exe` |
| Debian/Ubuntu     | `Printseca_x.y.z_amd64.deb`   |
| Fedora/openSUSE   | `Printseca-x.y.z-1.x86_64.rpm` |
| Linux (genérico)  | `Printseca_x.y.z_amd64.AppImage` |

No macOS, como o app não é assinado pela Apple, o macOS pede uma autorização
na primeira abertura — o passo a passo vem no `COMO-ABRIR-NO-MAC.txt` dentro
do zip.

### Desenvolvimento

Pré-requisitos: [Rust](https://rustup.rs/), Node.js, e as
[dependências de sistema do Tauri](https://v2.tauri.app/start/prerequisites/).

```sh
npm install
npm run tauri dev      # roda em modo dev
npm run tauri build    # gera o pacote da plataforma atual
```

---

## English

Desktop app that **reminds you (and helps) to print** a maintenance page every N
days, to keep the printer ink from drying out/clogging from disuse — a common and
expensive problem on cartridge and ink-tank printers.

It lives in the **system tray**, starts with the system and, when the interval is
due, either **notifies you** or **prints on its own** a page that exercises every
ink (cyan, magenta, yellow and black) or just black.

### Status

🚧 Early development.

### Stack

- **[Tauri v2](https://v2.tauri.app/)** (Rust + web frontend)
- Frontend: TypeScript + Vite (vanilla)
- Printing: CUPS (`lp`) on macOS/Linux; **SumatraPDF** sidecar on Windows

### Target platforms

| OS       | Format                              | Architecture    |
| -------- | ----------------------------------- | --------------- |
| macOS    | `.zip` with the ready-to-run app (unsigned — open guide inside the zip) | Apple Silicon   |
| Windows  | `.exe` (NSIS)                       | x64             |
| Linux    | `.AppImage` / `.deb` / `.rpm`       | x64             |

### Download

The packages are built automatically by GitHub Actions and live under the
repository **Releases**. Download the format for your platform:

| Platform          | File                          |
| ----------------- | ----------------------------- |
| macOS             | `Printseca-x.y.z-macos.zip`   |
| Windows           | `Printseca_x.y.z_x64-setup.exe` |
| Debian/Ubuntu     | `Printseca_x.y.z_amd64.deb`   |
| Fedora/openSUSE   | `Printseca-x.y.z-1.x86_64.rpm` |
| Linux (generic)   | `Printseca_x.y.z_amd64.AppImage` |

On macOS, since the app isn't signed by Apple, macOS asks for an authorization
the first time you open it — the step-by-step is in `HOW-TO-OPEN-ON-MAC.txt`
inside the zip.

### Development

Prerequisites: [Rust](https://rustup.rs/), Node.js, and the
[Tauri system dependencies](https://v2.tauri.app/start/prerequisites/).

```sh
npm install
npm run tauri dev      # runs in dev mode
npm run tauri build    # builds the package for the current platform
```
