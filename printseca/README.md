# Printseca

App de desktop que **lembra (e ajuda) a imprimir** uma página de manutenção a cada N
dias, para evitar que a tinta da impressora resseque/entupa por desuso — problema
comum e caro em impressoras de cartucho e de tanque de tinta.

Fica na **bandeja do sistema** (system tray), inicia junto com o sistema e, no
vencimento do intervalo, **notifica** ou **imprime sozinho** uma página que exercita
todas as tintas (ciano, magenta, amarelo e preto) ou apenas preto.

## Status

🚧 Em desenvolvimento inicial.

## Stack

- **[Tauri v2](https://v2.tauri.app/)** (Rust + frontend web)
- Frontend: TypeScript + Vite (vanilla)
- Impressão: CUPS (`lp`) no macOS/Linux; sidecar **SumatraPDF** no Windows

## Plataformas-alvo

| SO       | Formato                        | Arquitetura     |
| -------- | ------------------------------ | --------------- |
| macOS    | `.dmg` (assinado + notarizado) | Apple Silicon   |
| Windows  | `.exe` (NSIS)                  | x64             |
| Linux    | `.AppImage` / `.deb` / `.rpm`  | x64             |

## Desenvolvimento

Pré-requisitos: [Rust](https://rustup.rs/), Node.js, e as
[dependências de sistema do Tauri](https://v2.tauri.app/start/prerequisites/).

```sh
npm install
npm run tauri dev      # roda em modo dev
npm run tauri build    # gera o pacote da plataforma atual
```
