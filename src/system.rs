use crate::translations::Language;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents a Linux package manager configuration.
pub struct PackageManager {
    pub name: &'static str,
    pub install_cmd: &'static str,
    pub args: Vec<&'static str>,
}

/// Helper in pure Rust to check if an executable command exists in the system's PATH environment variable.
pub fn command_exists(cmd: &str) -> bool {
    if let Ok(path_env) = std::env::var("PATH") {
        for path_dir in std::env::split_paths(&path_env) {
            let cmd_path = path_dir.join(cmd);
            if cmd_path.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = cmd_path.metadata() {
                        if metadata.permissions().mode() & 0o111 != 0 {
                            return true;
                        }
                    }
                }
                #[cfg(not(unix))]
                return true;
            }
        }
    }
    false
}

/// Detects the available native package manager on the host system.
pub fn get_package_manager() -> Option<PackageManager> {
    if command_exists("pacman") {
        return Some(PackageManager {
            name: "Arch Linux (pacman)",
            install_cmd: "pacman",
            args: vec!["-S", "--needed", "niri"],
        });
    }
    if command_exists("dnf") {
        return Some(PackageManager {
            name: "Fedora (dnf)",
            install_cmd: "dnf",
            args: vec!["install", "-y", "niri"],
        });
    }
    if command_exists("zypper") {
        return Some(PackageManager {
            name: "openSUSE (zypper)",
            install_cmd: "zypper",
            args: vec!["install", "-y", "niri"],
        });
    }
    if command_exists("apt-get") {
        return Some(PackageManager {
            name: "Debian/Ubuntu (apt-get)",
            install_cmd: "apt-get",
            args: vec!["install", "-y", "niri"],
        });
    }
    None
}

/// Detects the language environment (Spanish or English fallback).
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

/// Resolves the absolute path to the Niri configuration file.
pub fn get_config_path(cli_path: &Option<PathBuf>) -> PathBuf {
    if let Some(path) = cli_path {
        return path.clone();
    }
    if let Some(mut path) = dirs::config_dir() {
        path.push("niri/config.kdl");
        path
    } else {
        let mut path = dirs::home_dir().expect("Could not detect HOME directory");
        path.push(".config/niri/config.kdl");
        path
    }
}

/// Validates the Niri KDL config file by calling 'niri validate'.
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
                    "Unknown validation error.".to_string()
                } else {
                    combined
                })
            }
        }
        Err(e) => Err(format!(
            "Could not execute 'niri validate'. Is Niri installed? Detail: {}",
            e
        )),
    }
}

/// Validates the Noctalia TOML config file by calling 'noctalia config validate'.
pub fn validate_noctalia_config(path: &Path) -> Result<(), String> {
    let output = Command::new("noctalia")
        .arg("config")
        .arg("validate")
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
                    "Unknown validation error.".to_string()
                } else {
                    combined
                })
            }
        }
        Err(e) => Err(format!(
            "Could not execute 'noctalia config validate'. Is Noctalia installed? Detail: {}",
            e
        )),
    }
}

/// Normalizes key combination string by sorting modifiers alphabetically.
pub fn normalize_key(key: &str) -> String {
    let mut parts: Vec<String> = key.split('+').map(|s| s.trim().to_string()).collect();
    parts.sort();
    parts.join("+")
}

/// Normalizes action text string by stripping quotes, semicolons, and collapsing spaces.
pub fn normalize_action(action: &str) -> String {
    let cleaned = action
        .replace('"', "")
        .replace(';', "")
        .replace('\n', " ")
        .replace('\r', "");
    
    // Collapse duplicate whitespaces
    let mut result = String::new();
    let mut last_was_space = false;
    for c in cleaned.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_key() {
        assert_eq!(normalize_key("Mod+Shift+Slash"), "Mod+Shift+Slash");
        assert_eq!(normalize_key("Shift+Mod+Slash"), "Mod+Shift+Slash");
        assert_eq!(normalize_key("Ctrl+Alt+Mod+Space"), "Alt+Ctrl+Mod+Space");
    }

    #[test]
    fn test_normalize_action() {
        assert_eq!(normalize_action("spawn \"ghostty\";"), "spawn ghostty");
        assert_eq!(normalize_action("spawn   \"ghostty\"   "), "spawn ghostty");
        assert_eq!(normalize_action("focus-workspace-down;"), "focus-workspace-down");
        assert_eq!(normalize_action("focus-workspace-down"), "focus-workspace-down");
    }
}
