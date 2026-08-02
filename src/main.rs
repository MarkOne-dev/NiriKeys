use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::process::Command;
use std::time::Duration;

mod app;
mod cli;
mod default_config;
mod system;
mod translations;
mod ui;

use app::{ActiveScreen, App, get_default_keybindings};
use cli::Args;
use system::{detect_language, get_config_path, get_package_manager, command_exists};
use translations::{Language, Translations};
use ui::ui_draw;

/// Entry point of the Nirikeys TUI application.
fn main() -> io::Result<()> {
    let args = Args::parse();
    let config_path = get_config_path(&args.config);
    let lang = detect_language();

    let mut app = App::new(config_path, args.dry_run, lang);
    if let Err(e) = app.init() {
        eprintln!("{}: {}", Translations::get(&app.lang).msg_fatal_init, e);
        std::process::exit(1);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut progress = 0;
    while progress <= 100 {
        let msg = match app.lang {
            Language::Es => match progress {
                0..=25 => "Detectando idioma del sistema...",
                26..=50 => "Buscando dependencias de Niri...",
                51..=75 => "Cargando archivo de configuración...",
                _ => "Validando sintaxis...",
            },
            Language::En => match progress {
                0..=25 => "Detecting system language...",
                26..=50 => "Checking Niri dependencies...",
                51..=75 => "Loading configuration file...",
                _ => "Validating syntax...",
            },
        };

        app.active_screen = ActiveScreen::Loading {
            progress,
            status_msg: msg.to_string(),
        };

        terminal.draw(|f| ui_draw(f, &mut app))?;
        std::thread::sleep(Duration::from_millis(50));
        progress += 5;
    }

    let niri_installed = command_exists("niri");

    if !niri_installed {
        if let Some(pm) = get_package_manager() {
            app.active_screen = ActiveScreen::InstallPrompt {
                pm_name: pm.name.to_string(),
                cmd: format!("sudo {} {}", pm.install_cmd, pm.args.join(" ")),
            };
        } else {
            let err_msg = match app.lang {
                Language::Es => "Niri no está instalado y no se detectó ningún gestor de paquetes soportado (pacman, dnf, zypper, apt). Por favor, instala Niri manualmente.".to_string(),
                Language::En => "Niri is not installed and no supported package manager was detected (pacman, dnf, zypper, apt). Please install Niri manually.".to_string(),
            };
            app.active_screen = ActiveScreen::ErrorPopup(err_msg);
        }
    } else {
        if !app.config_path.exists() {
            app.active_screen = ActiveScreen::CreateConfigPrompt;
        } else {
            app.active_screen = ActiveScreen::Dashboard;
        }
    }

    let res = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        match app.lang {
            Language::Es => eprint!("Ocurrió un error al ejecutar la TUI: {:?}", e),
            Language::En => eprint!("An error occurred while running the TUI: {:?}", e),
        }
    } else {
        println!("{}", Translations::get(&app.lang).msg_bye);
    }

    Ok(())
}

/// The main application event processing loop.
fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui_draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let active_screen = app.active_screen.clone();
                    match &active_screen {
                        ActiveScreen::Dashboard => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('1') => app.active_tab = 0,
                            KeyCode::Char('2') => app.active_tab = 1,
                            KeyCode::Char('3') => app.active_tab = 2,
                            KeyCode::Char('[') => {
                                app.preview_scroll = app.preview_scroll.saturating_sub(1);
                            }
                            KeyCode::Char(']') => {
                                app.preview_scroll = app.preview_scroll.saturating_add(1);
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.active_tab == 0 {
                                    app.move_selection_up();
                                } else if app.active_tab == 1 {
                                    let current = app.appearance_state.selected().unwrap_or(0);
                                    let settings_len = app.get_appearance_settings().len();
                                    if settings_len > 0 {
                                        let next = if current == 0 {
                                            settings_len - 1
                                        } else {
                                            current - 1
                                        };
                                        app.appearance_state.select(Some(next));
                                    }
                                } else {
                                    let current = app.noctalia_state.selected().unwrap_or(0);
                                    let settings_len = app.noctalia_settings.len();
                                    if settings_len > 0 {
                                        let next = if current == 0 {
                                            settings_len - 1
                                        } else {
                                            current - 1
                                        };
                                        app.noctalia_state.select(Some(next));
                                    }
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.active_tab == 0 {
                                    app.move_selection_down();
                                } else if app.active_tab == 1 {
                                    let current = app.appearance_state.selected().unwrap_or(0);
                                    let settings_len = app.get_appearance_settings().len();
                                    if settings_len > 0 {
                                        let next = if current >= settings_len - 1 {
                                            0
                                        } else {
                                            current + 1
                                        };
                                        app.appearance_state.select(Some(next));
                                    }
                                } else {
                                    let current = app.noctalia_state.selected().unwrap_or(0);
                                    let settings_len = app.noctalia_settings.len();
                                    if settings_len > 0 {
                                        let next = if current >= settings_len - 1 {
                                            0
                                        } else {
                                            current + 1
                                        };
                                        app.noctalia_state.select(Some(next));
                                    }
                                }
                            }
                            KeyCode::Char('a') => {
                                if app.active_tab == 0 {
                                    app.enter_add_mode();
                                }
                            }
                            KeyCode::Char('d') => {
                                if app.active_tab == 0 {
                                    app.delete_selected();
                                }
                            }
                            KeyCode::Char('b') => app.trigger_backup(),
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                if app.active_tab == 0 {
                                    let defaults = get_default_keybindings();
                                    let mut missing = Vec::new();
                                    for (default_key, default_action) in defaults {
                                        if let Some((_, user_action)) =
                                            app.keybindings.iter().find(|(k, _)| k == &default_key)
                                        {
                                            if default_action.trim() != user_action.trim() {
                                                missing.push((default_key, default_action));
                                            }
                                        } else {
                                            missing.push((default_key, default_action));
                                        }
                                    }
                                    if missing.is_empty() {
                                        app.active_screen = ActiveScreen::InfoPopup(match app.lang {
                                            Language::Es => "No se encontraron atajos faltantes en la plantilla oficial.".to_string(),
                                            Language::En => "No missing shortcuts from the official template were found.".to_string(),
                                        });
                                    } else {
                                        app.active_screen = ActiveScreen::MergePopup {
                                            missing,
                                            selected_idx: 0,
                                        };
                                    }
                                }
                            }
                            KeyCode::Enter | KeyCode::Char('e') | KeyCode::Char('E') => {
                                if app.active_tab == 1 {
                                    let settings = app.get_appearance_settings();
                                    if let Some(selected_idx) = app.appearance_state.selected() {
                                        if selected_idx < settings.len() {
                                            let setting = &settings[selected_idx];
                                            app.active_screen = ActiveScreen::EditAppearancePopup {
                                                setting_id: setting.id.clone(),
                                                setting_name: setting.name.clone(),
                                                input_value: if setting.value == "Default" {
                                                    String::new()
                                                } else {
                                                    setting.value.clone()
                                                },
                                            };
                                        }
                                    }
                                } else if app.active_tab == 2 {
                                    if let Some(selected_idx) = app.noctalia_state.selected() {
                                        if selected_idx < app.noctalia_settings.len() {
                                            let setting = &app.noctalia_settings[selected_idx];
                                            app.active_screen = ActiveScreen::EditNoctaliaPopup {
                                                setting_id: setting.id.clone(),
                                                setting_name: setting.name.clone(),
                                                input_value: if setting.value == "Default" {
                                                    String::new()
                                                } else {
                                                    setting.value.clone()
                                                },
                                                value_type: setting.value_type,
                                            };
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },
                        ActiveScreen::InstallPrompt { .. } => match key.code {
                            KeyCode::Char('i') | KeyCode::Char('I') => {
                                disable_raw_mode()?;
                                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                                terminal.show_cursor()?;

                                println!(
                                    "{}",
                                    match app.lang {
                                        Language::Es => "\n🔹 Iniciando instalación de Niri...",
                                        Language::En => "\n🔹 Starting Niri installation...",
                                    }
                                );

                                let install_result = if let Some(pm) = get_package_manager() {
                                    let status = Command::new("sudo")
                                        .arg(pm.install_cmd)
                                        .args(&pm.args)
                                        .status();

                                    match status {
                                        Ok(s) if s.success() => Ok(()),
                                        Ok(s) => Err(match app.lang {
                                            Language::Es => format!(
                                                "La instalación falló con código de salida: {:?}",
                                                s.code()
                                            ),
                                            Language::En => format!(
                                                "Installation failed with exit code: {:?}",
                                                s.code()
                                            ),
                                        }),
                                        Err(e) => Err(match app.lang {
                                            Language::Es => {
                                                format!("No se pudo ejecutar el comando: {}", e)
                                            }
                                            Language::En => {
                                                format!("Failed to execute command: {}", e)
                                            }
                                        }),
                                    }
                                } else {
                                    Err(match app.lang {
                                        Language::Es => {
                                            "No se detectó ningún gestor de paquetes.".to_string()
                                        }
                                        Language::En => "No package manager detected.".to_string(),
                                    })
                                };

                                enable_raw_mode()?;
                                execute!(io::stdout(), EnterAlternateScreen)?;
                                terminal.clear()?;

                                match install_result {
                                    Ok(_) => {
                                        if command_exists("niri") {
                                            app.update_metadata();
                                            app.active_screen = ActiveScreen::InfoPopup(match app
                                                .lang
                                            {
                                                Language::Es => {
                                                    "✔ Niri se instaló correctamente.".to_string()
                                                }
                                                Language::En => {
                                                    "✔ Niri was installed successfully.".to_string()
                                                }
                                            });
                                        } else {
                                            app.active_screen = ActiveScreen::ErrorPopup(match app.lang {
                                                Language::Es => "La instalación se completó, pero el ejecutable 'niri' no se encuentra en el PATH.".to_string(),
                                                Language::En => "Installation completed, but 'niri' executable is not found in PATH.".to_string(),
                                            });
                                        }
                                    }
                                    Err(err) => {
                                        app.active_screen = ActiveScreen::ErrorPopup(err);
                                    }
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break,
                            _ => {}
                        },
                        ActiveScreen::MergePopup {
                            missing,
                            selected_idx,
                        } => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                let mut idx = *selected_idx;
                                let len = missing.len();
                                if len > 0 {
                                    if idx == 0 {
                                        idx = len - 1;
                                    } else {
                                        idx -= 1;
                                    }
                                    app.active_screen = ActiveScreen::MergePopup {
                                        missing: missing.clone(),
                                        selected_idx: idx,
                                    };
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let mut idx = *selected_idx;
                                let len = missing.len();
                                if len > 0 {
                                    if idx >= len - 1 {
                                        idx = 0;
                                    } else {
                                        idx += 1;
                                    }
                                    app.active_screen = ActiveScreen::MergePopup {
                                        missing: missing.clone(),
                                        selected_idx: idx,
                                    };
                                }
                            }
                            KeyCode::Enter | KeyCode::Char('i') | KeyCode::Char('I') => {
                                let mut missing_clone = missing.clone();
                                let selected = missing_clone.remove(*selected_idx);
                                match app.apply_keybindings_batch(vec![selected]) {
                                    Ok(_) => {
                                        if missing_clone.is_empty() {
                                            app.active_screen = ActiveScreen::InfoPopup(match app
                                                .lang
                                            {
                                                Language::Es => {
                                                    "Atajo importado con éxito.".to_string()
                                                }
                                                Language::En => {
                                                    "Shortcut imported successfully.".to_string()
                                                }
                                            });
                                        } else {
                                            let next_idx = if *selected_idx >= missing_clone.len() {
                                                missing_clone.len() - 1
                                            } else {
                                                *selected_idx
                                            };
                                            app.active_screen = ActiveScreen::MergePopup {
                                                missing: missing_clone,
                                                selected_idx: next_idx,
                                            };
                                        }
                                    }
                                    Err(e) => {
                                        app.active_screen = ActiveScreen::ErrorPopup(e);
                                    }
                                }
                            }
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                let all_to_import = missing.clone();
                                match app.apply_keybindings_batch(all_to_import) {
                                    Ok(_) => {
                                        app.active_screen = ActiveScreen::InfoPopup(match app.lang {
                                            Language::Es => "✔ Todos los atajos sugeridos se importaron con éxito.".to_string(),
                                            Language::En => "✔ All recommended shortcuts were imported successfully.".to_string(),
                                        });
                                    }
                                    Err(e) => {
                                        app.active_screen = ActiveScreen::ErrorPopup(e);
                                    }
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                                app.active_screen = ActiveScreen::Dashboard;
                            }
                            _ => {}
                        },
                        ActiveScreen::CreateConfigPrompt => match key.code {
                            KeyCode::Char('y')
                            | KeyCode::Char('S')
                            | KeyCode::Char('s')
                            | KeyCode::Enter => {
                                if let Err(e) = app.create_default_config() {
                                    app.active_screen = ActiveScreen::ErrorPopup(e);
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => break,
                            _ => {}
                        },
                        ActiveScreen::AddPopup => match key.code {
                            KeyCode::Esc => app.active_screen = ActiveScreen::Dashboard,
                            KeyCode::Tab => app.toggle_input_focus(),
                            KeyCode::Enter => app.submit_add_form(),
                            KeyCode::Backspace => app.handle_backspace(),
                            KeyCode::Char(c) => app.handle_char(c),
                            _ => {}
                        },
                        ActiveScreen::ConfirmOverwrite {
                            key: k,
                            action: act,
                        } => match key.code {
                            KeyCode::Char('y')
                            | KeyCode::Char('S')
                            | KeyCode::Char('s')
                            | KeyCode::Enter => {
                                let k_clone = k.clone();
                                let act_clone = act.clone();
                                app.apply_keybinding(k_clone, act_clone);
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.active_screen = ActiveScreen::Dashboard;
                            }
                            _ => {}
                        },
                        ActiveScreen::EditAppearancePopup {
                            setting_id,
                            setting_name,
                            input_value,
                        } => match key.code {
                            KeyCode::Esc => app.active_screen = ActiveScreen::Dashboard,
                            KeyCode::Backspace => {
                                let mut val = input_value.clone();
                                val.pop();
                                app.active_screen = ActiveScreen::EditAppearancePopup {
                                    setting_id: setting_id.clone(),
                                    setting_name: setting_name.clone(),
                                    input_value: val,
                                };
                            }
                            KeyCode::Char(c) => {
                                let mut val = input_value.clone();
                                val.push(c);
                                app.active_screen = ActiveScreen::EditAppearancePopup {
                                    setting_id: setting_id.clone(),
                                    setting_name: setting_name.clone(),
                                    input_value: val,
                                };
                            }
                            KeyCode::Enter => {
                                let sid = setting_id.clone();
                                let val = input_value.trim().to_string();
                                match app.update_appearance_setting(&sid, val) {
                                    Ok(_) => {
                                        app.active_screen = ActiveScreen::InfoPopup(
                                            Translations::get(&app.lang)
                                                .msg_appearance_success
                                                .to_string(),
                                        );
                                    }
                                    Err(e) => {
                                        app.active_screen = ActiveScreen::ErrorPopup(e);
                                    }
                                }
                            }
                            _ => {}
                        },
                        ActiveScreen::EditNoctaliaPopup {
                            setting_id,
                            setting_name,
                            input_value,
                            value_type,
                        } => match key.code {
                            KeyCode::Esc => app.active_screen = ActiveScreen::Dashboard,
                            KeyCode::Backspace => {
                                let mut val = input_value.clone();
                                val.pop();
                                app.active_screen = ActiveScreen::EditNoctaliaPopup {
                                    setting_id: setting_id.clone(),
                                    setting_name: setting_name.clone(),
                                    input_value: val,
                                    value_type: *value_type,
                                };
                            }
                            KeyCode::Char(c) => {
                                let mut val = input_value.clone();
                                val.push(c);
                                app.active_screen = ActiveScreen::EditNoctaliaPopup {
                                    setting_id: setting_id.clone(),
                                    setting_name: setting_name.clone(),
                                    input_value: val,
                                    value_type: *value_type,
                                };
                            }
                            KeyCode::Enter => {
                                let sid = setting_id.clone();
                                let val = input_value.trim().to_string();
                                match app.update_noctalia_setting(&sid, val) {
                                    Ok(_) => {
                                        app.active_screen = ActiveScreen::InfoPopup(
                                            Translations::get(&app.lang)
                                                .msg_noctalia_success
                                                .to_string(),
                                        );
                                    }
                                    Err(e) => {
                                        app.active_screen = ActiveScreen::ErrorPopup(e);
                                    }
                                }
                            }
                            _ => {}
                        },
                        ActiveScreen::ErrorPopup(_) | ActiveScreen::InfoPopup(_) => {
                            app.active_screen = ActiveScreen::Dashboard;
                        }
                        ActiveScreen::Loading { .. } => {}
                    }
                }
            }
        }
    }
    Ok(())
}
