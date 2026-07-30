use crate::translations::Language;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct PackageManager {
    pub name: &'static str,
    pub install_cmd: &'static str,
    pub args: Vec<&'static str>,
}

pub fn get_package_manager() -> Option<PackageManager> {
    // 1. Verificar pacman (Arch Linux)
    if Command::new("which")
        .arg("pacman")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Some(PackageManager {
            name: "Arch Linux (pacman)",
            install_cmd: "pacman",
            args: vec!["-S", "--needed", "niri"],
        });
    }
    // 2. Verificar dnf (Fedora)
    if Command::new("which")
        .arg("dnf")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Some(PackageManager {
            name: "Fedora (dnf)",
            install_cmd: "dnf",
            args: vec!["install", "-y", "niri"],
        });
    }
    // 3. Verificar zypper (openSUSE)
    if Command::new("which")
        .arg("zypper")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Some(PackageManager {
            name: "openSUSE (zypper)",
            install_cmd: "zypper",
            args: vec!["install", "-y", "niri"],
        });
    }
    // 4. Verificar apt-get (Ubuntu/Debian)
    if Command::new("which")
        .arg("apt-get")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Some(PackageManager {
            name: "Debian/Ubuntu (apt-get)",
            install_cmd: "apt-get",
            args: vec!["install", "-y", "niri"],
        });
    }
    None
}

pub fn detect_language() -> Language {
    let lang_env = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .unwrap_or_else(|_| "en".to_string());

    if lang_env.to_lowercase().starts_with("es") {
        Language::Es
    } else {
        Language::En
    }
}

pub fn get_config_path(cli_path: &Option<PathBuf>) -> PathBuf {
    if let Some(path) = cli_path {
        return path.clone();
    }
    if let Some(mut path) = dirs::config_dir() {
        path.push("niri/config.kdl");
        path
    } else {
        let mut path = dirs::home_dir().expect("No se pudo detectar el directorio HOME");
        path.push(".config/niri/config.kdl");
        path
    }
}

pub fn validate_config(path: &Path) -> Result<(), String> {
    let output = Command::new("niri")
        .arg("validate")
        .arg("--config")
        .arg(path)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let combined = format!("{}{}", stdout, stderr);
                Err(if combined.trim().is_empty() {
                    "Error de validación desconocido.".to_string()
                } else {
                    combined
                })
            }
        }
        Err(e) => Err(format!(
            "No se pudo ejecutar 'niri validate'. ¿Está Niri instalado? Detalle: {}",
            e
        )),
    }
}
