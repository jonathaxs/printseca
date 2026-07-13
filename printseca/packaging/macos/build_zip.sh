#!/usr/bin/env bash
# Gera o .zip do Printseca para macOS: o app compilado (Printseca.app) mais um
# guia curto de como autorizar a abertura (o app não é assinado pela Apple).
#
# Por que zip com o .app (e não .dmg)? Assim distribuímos o app já pronto, sem o
# usuário precisar compilar do código-fonte. Como não temos Developer ID, uma
# assinatura "ad-hoc" (--sign -) evita o erro "o app está danificado" e deixa o
# macOS mostrar só o aviso normal de "desenvolvedor não identificado", que dá
# para liberar em Ajustes › Privacidade e Segurança (ver os guias .txt).
#
# Pré-requisitos: Node + Rust + toolchain do Tauri instalados.
# Uso (a partir de qualquer lugar):
#   bash packaging/macos/build_zip.sh
set -euo pipefail

APP_NAME="Printseca"

here="$(cd "$(dirname "$0")" && pwd)"   # printseca/packaging/macos
project="$(cd "$here/../.." && pwd)"    # printseca
# Pasta de saída dos artefatos de release. NÃO usamos `dist/` porque é lá que o
# Vite gera o frontend (frontendDist "../dist") — misturaria index.html/assets
# com os instaladores no upload do release.
dist="$project/artifacts"
app_build="$project/src-tauri/target/release/bundle/macos/$APP_NAME.app"

version="$(grep '"version"' "$project/src-tauri/tauri.conf.json" \
  | head -1 | sed -E 's/.*"version"[[:space:]]*:[[:space:]]*"([0-9.]+)".*/\1/')"

# Compila SÓ o .app (sem .dmg) — o `--bundles app` limita os alvos do bundler.
( cd "$project" && npm run tauri build -- --bundles app )

if [ ! -d "$app_build" ]; then
  echo "App não encontrado em $app_build" >&2
  exit 1
fi

# Assinatura ad-hoc: evita o "app está danificado" ao abrir um app sem Developer ID.
codesign --force --deep --sign - "$app_build"

# Monta a pasta do zip: o .app + os guias de abertura (pt-br e inglês).
staging="$(mktemp -d)"
trap 'rm -rf "$staging"' EXIT
foldername="$APP_NAME-$version-macos"
mkdir -p "$staging/$foldername"
cp -R "$app_build" "$staging/$foldername/"
cp "$here/COMO-ABRIR-NO-MAC.txt" "$staging/$foldername/"
cp "$here/HOW-TO-OPEN-ON-MAC.txt" "$staging/$foldername/"

mkdir -p "$dist"
out="$dist/$APP_NAME-$version-macos.zip"
rm -f "$out"
# -y preserva os symlinks internos do .app (frameworks), senão ele incha/quebra.
( cd "$staging" && zip -r -q -y "$out" "$foldername" )

echo "Zip gerado: $out"
