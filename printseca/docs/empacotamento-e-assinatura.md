# Empacotamento e assinatura

O workflow [`.github/workflows/release.yml`](../../.github/workflows/release.yml) gera os
instaladores das três plataformas e os publica num **release rascunho** do GitHub.

## Como disparar um release

```sh
# a partir de uma versão pronta:
git tag v0.1.0
git push origin v0.1.0
```

Ou manualmente em **Actions › Release › Run workflow**. Ao terminar, os instaladores
ficam nos _assets_ do release (em rascunho — revise e publique):

| Plataforma | Arquivo                        |
| ---------- | ------------------------------ |
| macOS      | `.dmg` (Apple Silicon)         |
| Windows    | `.exe` (NSIS, x64)             |
| Linux      | `.AppImage`, `.deb`, `.rpm`    |

> Sem os secrets de assinatura, o build **funciona** mas sai **sem assinar**: no macOS
> abre com botão direito › _Abrir_; no Windows o SmartScreen mostra um aviso.

## Assinatura + notarização do macOS (recomendado)

Você já tem o Apple Developer Program, então isso sai **sem custo extra**. São 6 secrets
no repositório (**Settings › Secrets and variables › Actions**):

| Secret                       | O que é                                                                 |
| ---------------------------- | ----------------------------------------------------------------------- |
| `APPLE_CERTIFICATE`          | seu certificado **Developer ID Application** (.p12) em **base64**        |
| `APPLE_CERTIFICATE_PASSWORD` | a senha que você definiu ao exportar o .p12                             |
| `APPLE_SIGNING_IDENTITY`     | ex.: `Developer ID Application: Seu Nome (TEAMID)`                       |
| `APPLE_ID`                   | o e-mail da sua conta Apple Developer                                    |
| `APPLE_PASSWORD`             | uma **senha de app** (appleid.apple.com › Segurança › Senhas de app)    |
| `APPLE_TEAM_ID`              | seu Team ID (10 caracteres, em developer.apple.com › Membership)        |

### Passo a passo do certificado

1. No **Xcode › Settings › Accounts**, ou em developer.apple.com, gere um certificado
   **Developer ID Application** (não confundir com o de App Store).
2. No app **Acesso às Chaves** (Keychain), ache esse certificado, clique com o botão
   direito › **Exportar** → salve como `.p12` e defina uma senha
   (= `APPLE_CERTIFICATE_PASSWORD`).
3. Converta para base64 e copie para o clipboard:
   ```sh
   base64 -i certificado.p12 | pbcopy
   ```
   Cole o conteúdo no secret `APPLE_CERTIFICATE`.
4. Descubra a identidade exata para o `APPLE_SIGNING_IDENTITY`:
   ```sh
   security find-identity -v -p codesigning
   ```

Com esses secrets definidos, o `tauri-action` assina **e notariza** o `.dmg`
automaticamente — ele abre sem nenhum aviso do Gatekeeper.

## Windows (opcional)

O `.exe` sai sem assinatura por padrão (SmartScreen avisa, mas instala). Assinar exige
um certificado de _code signing_ pago de uma CA — fica para depois, se necessário.
