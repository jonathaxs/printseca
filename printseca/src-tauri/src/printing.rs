// ============================================================================
// printing.rs — Impressão e listagem de impressoras
//
// A estratégia muda por sistema operacional (usamos `#[cfg(target_os = ...)]`
// para compilar só o trecho certo em cada plataforma):
//   • macOS / Linux: usam o CUPS, então chamamos os comandos `lpstat` e `lp`.
//   • Windows: não tem CUPS; embutimos o programa SumatraPDF e o chamamos para
//     imprimir o PDF em silêncio.
//
// Detalhe importante: NÃO precisamos de uma "flag de cor". Como geramos dois
// PDFs (um colorido e um só preto), imprimir em P&B é só escolher o arquivo PB.
// ============================================================================

use std::path::PathBuf;
use std::process::Command;

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime};

/// Lista as impressoras disponíveis no sistema.
pub fn list_printers() -> Vec<String> {
    // `lpstat -e` lista os nomes das impressoras (CUPS).
    #[cfg(not(target_os = "windows"))]
    let output = Command::new("lpstat").arg("-e").output();

    // No Windows pedimos a lista ao PowerShell.
    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Printer | Select-Object -ExpandProperty Name",
        ])
        .output();

    // Pega a saída do comando, quebra em linhas, remove espaços e linhas vazias.
    // Se o comando falhar, devolvemos uma lista vazia (unwrap_or_default).
    output
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Caminho do PDF de manutenção (colorido ou P&B).
pub fn pdf_path<R: Runtime>(app: &AppHandle<R>, color: bool) -> Result<PathBuf, String> {
    let name = if color {
        "manutencao-cor.pdf"
    } else {
        "manutencao-pb.pdf"
    };

    // Em desenvolvimento (debug) os PDFs ficam em src-tauri/resources;
    // já no app instalado (release) eles vêm empacotados no "resource dir".
    // `env!("CARGO_MANIFEST_DIR")` é a pasta do projeto no momento da compilação.
    #[cfg(debug_assertions)]
    {
        let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(name);
        if dev.exists() {
            return Ok(dev);
        }
    }

    // Em release: o Tauri resolve o caminho do recurso empacotado.
    app.path()
        .resolve(format!("resources/{name}"), BaseDirectory::Resource)
        .map_err(|e| e.to_string())
}

/// Imprime o PDF de manutenção na impressora indicada (ou na padrão se None).
pub fn print_pdf<R: Runtime>(
    app: &AppHandle<R>,
    color: bool,
    printer: Option<&str>,
) -> Result<(), String> {
    let pdf = pdf_path(app, color)?;

    #[cfg(not(target_os = "windows"))]
    {
        // macOS / Linux: `lp [-d impressora] arquivo.pdf`
        let mut cmd = Command::new("lp");
        if let Some(p) = printer {
            cmd.arg("-d").arg(p); // -d escolhe a impressora; sem isso, usa a padrão
        }
        cmd.arg(&pdf);
        // `.status()` roda o comando e espera ele terminar.
        let status = cmd
            .status()
            .map_err(|e| format!("falha ao executar 'lp': {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("'lp' retornou erro (código {:?})", status.code()))
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: usamos o SumatraPDF.exe que é empacotado junto ao app.
        let sumatra = app
            .path()
            .resolve("SumatraPDF.exe", BaseDirectory::Resource)
            .map_err(|e| e.to_string())?;
        let mut cmd = Command::new(sumatra);
        match printer {
            Some(p) => {
                cmd.args(["-print-to", p]); // imprime numa impressora específica
            }
            None => {
                cmd.arg("-print-to-default"); // ou na impressora padrão
            }
        }
        cmd.arg("-silent").arg(&pdf); // -silent = sem abrir janela/diálogo
        let status = cmd
            .status()
            .map_err(|e| format!("falha ao executar SumatraPDF: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "SumatraPDF retornou erro (código {:?})",
                status.code()
            ))
        }
    }
}
