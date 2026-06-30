use std::path::PathBuf;
use std::process::Command;

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime};

/// Lista as impressoras disponíveis no sistema.
pub fn list_printers() -> Vec<String> {
    #[cfg(not(target_os = "windows"))]
    let output = Command::new("lpstat").arg("-e").output();

    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Printer | Select-Object -ExpandProperty Name",
        ])
        .output();

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

/// Caminho do PDF de manutenção (colorido ou PB).
pub fn pdf_path<R: Runtime>(app: &AppHandle<R>, color: bool) -> Result<PathBuf, String> {
    let name = if color {
        "manutencao-cor.pdf"
    } else {
        "manutencao-pb.pdf"
    };

    // Em dev usamos os arquivos diretamente de src-tauri/resources;
    // em release eles vêm empacotados no resource dir.
    #[cfg(debug_assertions)]
    {
        let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(name);
        if dev.exists() {
            return Ok(dev);
        }
    }

    app.path()
        .resolve(format!("resources/{name}"), BaseDirectory::Resource)
        .map_err(|e| e.to_string())
}

/// Imprime o PDF de manutenção na impressora indicada (ou na padrão).
pub fn print_pdf<R: Runtime>(
    app: &AppHandle<R>,
    color: bool,
    printer: Option<&str>,
) -> Result<(), String> {
    let pdf = pdf_path(app, color)?;

    #[cfg(not(target_os = "windows"))]
    {
        // macOS / Linux: CUPS
        let mut cmd = Command::new("lp");
        if let Some(p) = printer {
            cmd.arg("-d").arg(p);
        }
        cmd.arg(&pdf);
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
        // Windows: sidecar SumatraPDF empacotado junto ao app
        let sumatra = app
            .path()
            .resolve("SumatraPDF.exe", BaseDirectory::Resource)
            .map_err(|e| e.to_string())?;
        let mut cmd = Command::new(sumatra);
        match printer {
            Some(p) => {
                cmd.args(["-print-to", p]);
            }
            None => {
                cmd.arg("-print-to-default");
            }
        }
        cmd.arg("-silent").arg(&pdf);
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
