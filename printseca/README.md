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

| SO       | Formato                             | Arquitetura     |
| -------- | ----------------------------------- | --------------- |
| macOS    | rodar via código-fonte (sem assinatura, ver [COMO-RODAR-NO-MAC.txt](COMO-RODAR-NO-MAC.txt)) | Apple Silicon   |
| Windows  | `.exe` (NSIS) / `.msi`              | x64             |
| Linux    | `.AppImage` / `.deb` / `.rpm`       | x64             |

## Download (Windows e Linux)

Os instaladores são gerados automaticamente pelo GitHub Actions e ficam nos
**Releases** do repositório. Baixe o formato da sua plataforma:

| Plataforma        | Arquivo                       |
| ----------------- | ----------------------------- |
| Windows           | `Printseca_x.y.z_x64-setup.exe` |
| Debian/Ubuntu     | `Printseca_x.y.z_amd64.deb`   |
| Fedora/openSUSE   | `Printseca-x.y.z-1.x86_64.rpm` |
| Linux (genérico)  | `Printseca_x.y.z_amd64.AppImage` |

No macOS não há instalador: roda direto do código-fonte seguindo o
[COMO-RODAR-NO-MAC.txt](COMO-RODAR-NO-MAC.txt).

## Desenvolvimento

Pré-requisitos: [Rust](https://rustup.rs/), Node.js, e as
[dependências de sistema do Tauri](https://v2.tauri.app/start/prerequisites/).

```sh
npm install
npm run tauri dev      # roda em modo dev
npm run tauri build    # gera o pacote da plataforma atual
```
