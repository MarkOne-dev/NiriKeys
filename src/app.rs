use std::fs;
use std::path::{Path, PathBuf};
use kdl::KdlDocument;
use ratatui::widgets::ListState;
use tempfile::NamedTempFile;
use crate::translations::{Language, Translations};
use crate::default_config;
use crate::system::validate_config;

#[derive(Clone, Debug)]
pub enum ActiveScreen {
    Loading { progress: u16, status_msg: String },
    InstallPrompt { pm_name: String, cmd: String },
    Dashboard,
    AddPopup,
    ConfirmOverwrite { key: String, action: String },
    ErrorPopup(String),
    InfoPopup(String),
    CreateConfigPrompt,
    MergePopup {
        missing: Vec<(String, String)>,
        selected_idx: usize,
    },
    EditAppearancePopup {
        setting_id: String,
        setting_name: String,
        input_value: String,
    },
}

#[derive(Clone, Debug)]
pub struct AppearanceSetting {
    pub id: String,
    pub name: String,
    pub value: String,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InputFocus {
    Key,
    Action,
}

pub struct App {
    pub config_path: PathBuf,
    pub dry_run: bool,
    pub lang: Language,
    pub doc: Option<KdlDocument>,
    pub keybindings: Vec<(String, String)>,
    pub list_state: ListState,
    pub active_screen: ActiveScreen,
    
    // Form Inputs
    pub input_key: String,
    pub input_action: String,
    pub input_focus: InputFocus,

    // File Metadata
    pub file_size_kb: f64,
    pub file_mod_time: String,
    pub file_is_valid: bool,

    // Tab Navigation & Appearance
    pub active_tab: usize, // 0 = Keybindings, 1 = Appearance
    pub appearance_state: ListState,
}

impl App {
    pub fn new(config_path: PathBuf, dry_run: bool, lang: Language) -> Self {
        Self {
            config_path,
            dry_run,
            lang,
            doc: None,
            keybindings: Vec::new(),
            list_state: ListState::default(),
            active_screen: ActiveScreen::Loading {
                progress: 0,
                status_msg: String::new(),
            },
            input_key: String::new(),
            input_action: String::new(),
            input_focus: InputFocus::Key,
            file_size_kb: 0.0,
            file_mod_time: String::new(),
            file_is_valid: false,
            active_tab: 0,
            appearance_state: ListState::default(),
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        if self.config_path.exists() {
            self.load_doc()?;
            self.update_metadata();
        }
        Ok(())
    }

    pub fn load_doc(&mut self) -> Result<(), String> {
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| match self.lang {
                Language::Es => format!("Error al leer archivo: {}", e),
                Language::En => format!("Error reading file: {}", e),
            })?;
        
        let parsed_doc: KdlDocument = content.parse()
            .map_err(|e| match self.lang {
                Language::Es => format!("Error al parsear KDL: {}", e),
                Language::En => format!("Error parsing KDL: {}", e),
            })?;
        
        self.doc = Some(parsed_doc);
        self.reload_keybindings()?;
        Ok(())
    }

    pub fn reload_keybindings(&mut self) -> Result<(), String> {
        let mut bindings = Vec::new();
        if let Some(ref doc) = self.doc {
            if let Some(binds_node) = doc.nodes().iter().find(|n| n.name().value() == "binds") {
                if let Some(children) = binds_node.children() {
                    for node in children.nodes() {
                        let key = node.name().value();
                        let action_desc = get_action_desc(node);
                        bindings.push((key.to_string(), action_desc));
                    }
                }
            }
        }
        self.keybindings = bindings;

        // Ajustar el índice de selección
        let len = self.keybindings.len();
        if len == 0 {
            self.list_state.select(None);
        } else {
            let curr = self.list_state.selected().unwrap_or(0);
            if curr >= len {
                self.list_state.select(Some(len - 1));
            } else {
                self.list_state.select(Some(curr));
            }
        }
        if self.appearance_state.selected().is_none() {
            self.appearance_state.select(Some(0));
        }
        Ok(())
    }

    pub fn update_metadata(&mut self) {
        if let Ok(metadata) = fs::metadata(&self.config_path) {
            self.file_size_kb = (metadata.len() as f64) / 1024.0;
            if let Ok(modified) = metadata.modified() {
                let datetime: chrono::DateTime<chrono::Local> = modified.into();
                self.file_mod_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
            }
        }
        self.file_is_valid = validate_config(&self.config_path).is_ok();
    }

    pub fn create_default_config(&mut self) -> Result<(), String> {
        let parent = self.config_path.parent()
            .ok_or_else(|| match self.lang {
                Language::Es => "No se pudo detectar el directorio raíz".to_string(),
                Language::En => "Could not detect parent directory".to_string(),
            })?;

        fs::create_dir_all(parent)
            .map_err(|e| match self.lang {
                Language::Es => format!("No se pudo crear directorio {}: {}", parent.display(), e),
                Language::En => format!("Could not create directory {}: {}", parent.display(), e),
            })?;

        fs::write(&self.config_path, default_config::DEFAULT_CONFIG)
            .map_err(|e| match self.lang {
                Language::Es => format!("Error al escribir archivo por defecto: {}", e),
                Language::En => format!("Error writing default file: {}", e),
            })?;

        self.load_doc()?;
        self.update_metadata();
        self.active_screen = ActiveScreen::Dashboard;
        Ok(())
    }

    // --- ACCIONES ---

    pub fn move_selection_up(&mut self) {
        if self.keybindings.is_empty() { return; }
        let current = self.list_state.selected().unwrap_or(0);
        let next = if current == 0 {
            self.keybindings.len() - 1
        } else {
            current - 1
        };
        self.list_state.select(Some(next));
    }

    pub fn move_selection_down(&mut self) {
        if self.keybindings.is_empty() { return; }
        let current = self.list_state.selected().unwrap_or(0);
        let next = if current >= self.keybindings.len() - 1 {
            0
        } else {
            current + 1
        };
        self.list_state.select(Some(next));
    }

    pub fn enter_add_mode(&mut self) {
        self.input_key.clear();
        self.input_action.clear();
        self.input_focus = InputFocus::Key;
        self.active_screen = ActiveScreen::AddPopup;
    }

    pub fn delete_selected(&mut self) {
        let selected = match self.list_state.selected() {
            Some(idx) => idx,
            Option::None => return,
        };

        let (key, _) = &self.keybindings[selected];
        let key_to_remove = key.clone();

        let doc = match self.doc.as_mut() {
            Some(d) => d,
            Option::None => return,
        };

        if let Some(binds_node) = doc.nodes_mut().iter_mut().find(|n| n.name().value() == "binds") {
            let children = binds_node.ensure_children();
            if let Some(pos) = children.nodes().iter().position(|n| n.name().value() == key_to_remove) {
                children.nodes_mut().remove(pos);
            }
        }

        if let Err(e) = self.save_and_validate_changes() {
            self.active_screen = ActiveScreen::ErrorPopup(e);
        } else {
            self.active_screen = ActiveScreen::InfoPopup(Translations::get(&self.lang).msg_deleted_success.to_string());
        }
    }

    pub fn trigger_backup(&mut self) {
        if !self.config_path.exists() {
            self.active_screen = ActiveScreen::ErrorPopup(match self.lang {
                Language::Es => "No existe archivo de configuración".to_string(),
                Language::En => "Configuration file does not exist".to_string(),
            });
            return;
        }

        let mut backup_path = self.config_path.clone();
        let ext = backup_path.extension()
            .map(|e| format!("{}.bak", e.to_string_lossy()))
            .unwrap_or_else(|| "bak".to_string());
        backup_path.set_extension(ext);

        match fs::copy(&self.config_path, &backup_path) {
            Ok(_) => {
                self.active_screen = ActiveScreen::InfoPopup(format!(
                    "{}", 
                    Translations::get(&self.lang).msg_backup_success.replace("{}", &backup_path.to_string_lossy())
                ));
            }
            Err(e) => {
                self.active_screen = ActiveScreen::ErrorPopup(format!(
                    "{}: {}",
                    Translations::get(&self.lang).msg_backup_err,
                    e
                ));
            }
        }
    }

    pub fn toggle_input_focus(&mut self) {
        self.input_focus = match self.input_focus {
            InputFocus::Key => InputFocus::Action,
            InputFocus::Action => InputFocus::Key,
        };
    }

    pub fn handle_backspace(&mut self) {
        match self.input_focus {
            InputFocus::Key => { self.input_key.pop(); }
            InputFocus::Action => { self.input_action.pop(); }
        }
    }

    pub fn handle_char(&mut self, c: char) {
        match self.input_focus {
            InputFocus::Key => { self.input_key.push(c); }
            InputFocus::Action => { self.input_action.push(c); }
        }
    }

    pub fn submit_add_form(&mut self) {
        let key = self.input_key.trim().to_string();
        let action = self.input_action.trim().to_string();

        if key.is_empty() || action.is_empty() {
            self.active_screen = ActiveScreen::ErrorPopup(Translations::get(&self.lang).msg_empty_fields.to_string());
            return;
        }

        // Verificar si ya existe duplicado
        let mut exists = false;
        if let Some(ref doc) = self.doc {
            if let Some(binds_node) = doc.nodes().iter().find(|n| n.name().value() == "binds") {
                if let Some(children) = binds_node.children() {
                    exists = children.nodes().iter().any(|n| n.name().value() == key);
                }
            }
        }

        if exists {
            self.active_screen = ActiveScreen::ConfirmOverwrite { key, action };
        } else {
            self.apply_keybinding(key, action);
        }
    }

    pub fn apply_keybinding(&mut self, key: String, action: String) {
        let doc = match self.doc.as_mut() {
            Some(d) => d,
            Option::None => return,
        };

        // Buscar o crear nodo binds
        let binds_node = if let Some(idx) = doc.nodes().iter().position(|node| node.name().value() == "binds") {
            &mut doc.nodes_mut()[idx]
        } else {
            let new_binds = kdl::KdlNode::new("binds");
            doc.nodes_mut().push(new_binds);
            let len = doc.nodes().len();
            &mut doc.nodes_mut()[len - 1]
        };

        let children = binds_node.ensure_children();

        // Eliminar duplicado si existe
        if let Some(pos) = children.nodes().iter().position(|n| n.name().value() == key) {
            children.nodes_mut().remove(pos);
        }

        // Dividir el comando de acción inteligentemente
        let formatted_action = if Self::is_niri_native_action(&action) {
            action
        } else if action.contains(' ') && !action.starts_with('"') {
            // Dividir y encorchetar argumentos
            let parts: Vec<&str> = action.split_whitespace().collect();
            let spawn_args = parts.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(" ");
            format!("spawn {}", spawn_args)
        } else if !action.starts_with('"') {
            format!("spawn \"{}\"", action)
        } else {
            action
        };

        // Generar nodo KDL
        let snippet = format!("    \"{}\" {{ {}; }}\n", key, formatted_action);
        let parsed_snippet = match snippet.parse::<KdlDocument>() {
            Ok(mut temp_doc) => temp_doc.nodes_mut().remove(0),
            Err(e) => {
                self.active_screen = ActiveScreen::ErrorPopup(match self.lang {
                    Language::Es => format!("KDL inválido: {}", e),
                    Language::En => format!("Invalid KDL: {}", e),
                });
                return;
            }
        };

        children.nodes_mut().push(parsed_snippet);

        if let Err(e) = self.save_and_validate_changes() {
            self.active_screen = ActiveScreen::ErrorPopup(e);
        } else {
            self.active_screen = ActiveScreen::InfoPopup(Translations::get(&self.lang).msg_save_success.to_string());
        }
    }

    pub fn apply_keybindings_batch(&mut self, new_bindings: Vec<(String, String)>) -> Result<(), String> {
        let doc = match self.doc.as_mut() {
            Some(d) => d,
            Option::None => return Err("KDL document not loaded".to_string()),
        };

        // Buscar o crear nodo binds
        let binds_node = if let Some(idx) = doc.nodes().iter().position(|node| node.name().value() == "binds") {
            &mut doc.nodes_mut()[idx]
        } else {
            let new_binds = kdl::KdlNode::new("binds");
            doc.nodes_mut().push(new_binds);
            let len = doc.nodes().len();
            &mut doc.nodes_mut()[len - 1]
        };

        let children = binds_node.ensure_children();

        for (key, action) in new_bindings {
            // Eliminar duplicado si existe
            if let Some(pos) = children.nodes().iter().position(|n| n.name().value() == key) {
                children.nodes_mut().remove(pos);
            }

            // Dividir el comando de acción inteligentemente
            let formatted_action = if Self::is_niri_native_action(&action) {
                action
            } else if action.contains(' ') && !action.starts_with('"') {
                let parts: Vec<&str> = action.split_whitespace().collect();
                let spawn_args = parts.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(" ");
                format!("spawn {}", spawn_args)
            } else if !action.starts_with('"') {
                format!("spawn \"{}\"", action)
            } else {
                action
            };

            // Generar nodo KDL
            let snippet = format!("    \"{}\" {{ {}; }}\n", key, formatted_action);
            let parsed_node = snippet.parse::<KdlDocument>()
                .map_err(|e| format!("KDL snippet parse error: {} for snippet: {}", e, snippet))?
                .nodes_mut().remove(0);
            
            children.nodes_mut().push(parsed_node);
        }

        self.save_and_validate_changes()?;
        Ok(())
    }

    pub fn save_and_validate_changes(&mut self) -> Result<(), String> {
        let doc = self.doc.as_ref().ok_or_else(|| match self.lang {
            Language::Es => "No hay ningún documento cargado".to_string(),
            Language::En => "No document loaded".to_string(),
        })?;
        let serialized = doc.to_string();

        if self.dry_run {
            self.reload_keybindings()?;
            self.update_metadata();
            return Ok(());
        }

        let parent = self.config_path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .map_err(|e| match self.lang {
                Language::Es => format!("No se pudo crear directorio parent: {}", e),
                Language::En => format!("Could not create parent directory: {}", e),
            })?;

        let mut temp_file = NamedTempFile::new_in(parent)
            .map_err(|e| match self.lang {
                Language::Es => format!("No se pudo crear archivo temporal: {}", e),
                Language::En => format!("Could not create temporary file: {}", e),
            })?;

        use std::io::Write;
        temp_file.write_all(serialized.as_bytes())
            .map_err(|e| match self.lang {
                Language::Es => format!("Error al escribir archivo temporal: {}", e),
                Language::En => format!("Error writing temporary file: {}", e),
            })?;

        match validate_config(temp_file.path()) {
            Ok(_) => {
                temp_file.persist(&self.config_path)
                    .map_err(|e| match self.lang {
                        Language::Es => format!("Error al guardar el archivo definitivo: {}", e),
                        Language::En => format!("Error saving final configuration file: {}", e),
                    })?;
                
                self.load_doc()?;
                self.update_metadata();
                Ok(())
            }
            Err(err_msg) => {
                // Revertir cambios en memoria recargando el archivo original
                let _ = self.load_doc();
                Err(err_msg)
            }
        }
    }

    fn is_niri_native_action(action: &str) -> bool {
        let first_word = action.split_whitespace().next().unwrap_or("");
        matches!(
            first_word,
            "spawn"
                | "spawn-sh"
                | "close-window"
                | "quit"
                | "focus-column-left"
                | "focus-column-right"
                | "focus-window-down"
                | "focus-window-up"
                | "move-column-left"
                | "move-column-right"
                | "move-window-down"
                | "move-window-up"
                | "focus-workspace"
                | "move-column-to-workspace"
                | "move-window-to-workspace"
                | "focus-monitor"
                | "move-column-to-monitor"
                | "move-window-to-monitor"
                | "move-workspace-to-monitor"
                | "toggle-column-width-largest"
                | "toggle-column-width-smallest"
                | "set-column-width"
                | "set-window-height"
                | "toggle-window-floating"
                | "toggle-fullscreen"
                | "toggle-window-maximized"
                | "screenshot"
                | "screenshot-screen"
                | "screenshot-window"
                | "show-hotkey-overlay"
                | "toggle-keyboard-layout"
                | "focus-workspace-down"
                | "focus-workspace-up"
                | "move-column-to-workspace-down"
                | "move-column-to-workspace-up"
                | "move-window-to-workspace-down"
                | "move-window-to-workspace-up"
        )
    }

    pub fn get_appearance_settings(&self) -> Vec<AppearanceSetting> {
        let mut settings = Vec::new();
        
        let doc = match &self.doc {
            Some(d) => d,
            Option::None => return settings,
        };

        // Helper to find layout node
        let layout_node = doc.nodes().iter().find(|n| n.name().value() == "layout");

        // Helper to find layout child setting
        let get_layout_val = |child_name: &str| -> String {
            if let Some(layout) = layout_node {
                if let Some(children) = layout.children() {
                    if let Some(child) = children.nodes().iter().find(|n| n.name().value() == child_name) {
                        if let Some(val) = child.entries().first() {
                            return val.value().to_string();
                        }
                    }
                }
            }
            "Default".to_string()
        };

        // Helper to find sub-node values (like focus-ring or border)
        let get_subnode_val = |node_name: &str, child_name: &str| -> String {
            if let Some(layout) = layout_node {
                if let Some(children) = layout.children() {
                    if let Some(sub) = children.nodes().iter().find(|n| n.name().value() == node_name) {
                        if let Some(sub_children) = sub.children() {
                            if let Some(child) = sub_children.nodes().iter().find(|n| n.name().value() == child_name) {
                                if let Some(val) = child.entries().first() {
                                    return val.value().to_string();
                                }
                            }
                        }
                    }
                }
            }
            "Default".to_string()
        };

        // Helper to check if a sub-node is active or has off/on
        let is_border_off = || -> bool {
            if let Some(layout) = layout_node {
                if let Some(children) = layout.children() {
                    if let Some(sub) = children.nodes().iter().find(|n| n.name().value() == "border") {
                        if let Some(sub_children) = sub.children() {
                            return sub_children.nodes().iter().any(|n| n.name().value() == "off");
                        }
                    }
                }
            }
            true // default off
        };

        // Helper to find geometry corner radius
        let get_corner_radius = || -> String {
            for node in doc.nodes() {
                if node.name().value() == "window-rule" {
                    if let Some(children) = node.children() {
                        if let Some(child) = children.nodes().iter().find(|n| n.name().value() == "geometry-corner-radius") {
                            if let Some(val) = child.entries().first() {
                                return val.value().to_string();
                            }
                        }
                    }
                }
            }
            "Default".to_string()
        };

        // 1. Gaps
        settings.push(AppearanceSetting {
            id: "gaps".to_string(),
            name: match self.lang {
                Language::Es => "Espaciado general (Gaps)".to_string(),
                Language::En => "General Gaps".to_string(),
            },
            value: get_layout_val("gaps"),
        });

        // 2. Focus Ring Width
        settings.push(AppearanceSetting {
            id: "focus_ring_width".to_string(),
            name: match self.lang {
                Language::Es => "Foco: Grosor del anillo".to_string(),
                Language::En => "Focus Ring: Width".to_string(),
            },
            value: get_subnode_val("focus-ring", "width"),
        });

        // 3. Focus Ring Active Color
        settings.push(AppearanceSetting {
            id: "focus_ring_active".to_string(),
            name: match self.lang {
                Language::Es => "Foco: Color activo".to_string(),
                Language::En => "Focus Ring: Active Color".to_string(),
            },
            value: get_subnode_val("focus-ring", "active-color"),
        });

        // 4. Focus Ring Inactive Color
        settings.push(AppearanceSetting {
            id: "focus_ring_inactive".to_string(),
            name: match self.lang {
                Language::Es => "Foco: Color inactivo".to_string(),
                Language::En => "Focus Ring: Inactive Color".to_string(),
            },
            value: get_subnode_val("focus-ring", "inactive-color"),
        });

        // 5. Border Status
        settings.push(AppearanceSetting {
            id: "border_status".to_string(),
            name: match self.lang {
                Language::Es => "Bordes de ventana (on/off)".to_string(),
                Language::En => "Window Borders (on/off)".to_string(),
            },
            value: if is_border_off() { "off".to_string() } else { "on".to_string() },
        });

        // 6. Border Width
        settings.push(AppearanceSetting {
            id: "border_width".to_string(),
            name: match self.lang {
                Language::Es => "Borde: Grosor".to_string(),
                Language::En => "Border: Width".to_string(),
            },
            value: get_subnode_val("border", "width"),
        });

        // 7. Border Active Color
        settings.push(AppearanceSetting {
            id: "border_active".to_string(),
            name: match self.lang {
                Language::Es => "Borde: Color activo".to_string(),
                Language::En => "Border: Active Color".to_string(),
            },
            value: get_subnode_val("border", "active-color"),
        });

        // 8. Border Inactive Color
        settings.push(AppearanceSetting {
            id: "border_inactive".to_string(),
            name: match self.lang {
                Language::Es => "Borde: Color inactivo".to_string(),
                Language::En => "Border: Inactive Color".to_string(),
            },
            value: get_subnode_val("border", "inactive-color"),
        });

        // 9. Corner Radius
        settings.push(AppearanceSetting {
            id: "corner_radius".to_string(),
            name: match self.lang {
                Language::Es => "Radio de esquinas (Geometry)".to_string(),
                Language::En => "Window Corner Radius".to_string(),
            },
            value: get_corner_radius(),
        });

        settings
    }

    pub fn update_appearance_setting(&mut self, id: &str, value: String) -> Result<(), String> {
        let doc = match self.doc.as_mut() {
            Some(d) => d,
            Option::None => return Err("KDL document not loaded".to_string()),
        };

        // 1. Get or create layout node
        let layout_idx = if let Some(idx) = doc.nodes().iter().position(|n| n.name().value() == "layout") {
            idx
        } else {
            let new_layout = kdl::KdlNode::new("layout");
            doc.nodes_mut().push(new_layout);
            doc.nodes().len() - 1
        };
        let layout = &mut doc.nodes_mut()[layout_idx];
        let layout_children = layout.ensure_children();

        match id {
            "gaps" => {
                if let Some(pos) = layout_children.nodes().iter().position(|n| n.name().value() == "gaps") {
                    layout_children.nodes_mut().remove(pos);
                }
                let snippet = format!("    gaps {}\n", value);
                let node = snippet.parse::<KdlDocument>()
                    .map_err(|e| format!("KDL parse error: {}", e))?
                    .nodes_mut().remove(0);
                layout_children.nodes_mut().push(node);
            }
            "focus_ring_width" | "focus_ring_active" | "focus_ring_inactive" => {
                let ring_idx = if let Some(idx) = layout_children.nodes().iter().position(|n| n.name().value() == "focus-ring") {
                    idx
                } else {
                    let new_ring = kdl::KdlNode::new("focus-ring");
                    layout_children.nodes_mut().push(new_ring);
                    layout_children.nodes().len() - 1
                };
                let ring = &mut layout_children.nodes_mut()[ring_idx];
                let ring_children = ring.ensure_children();

                let prop_name = match id {
                    "focus_ring_width" => "width",
                    "focus_ring_active" => "active-color",
                    _ => "inactive-color",
                };

                if let Some(pos) = ring_children.nodes().iter().position(|n| n.name().value() == prop_name) {
                    ring_children.nodes_mut().remove(pos);
                }

                let formatted_value = if prop_name.contains("color") {
                    if !value.starts_with('"') {
                        format!("\"{}\"", value)
                    } else {
                        value
                    }
                } else {
                    value
                };

                let snippet = format!("    {} {}\n", prop_name, formatted_value);
                let node = snippet.parse::<KdlDocument>()
                    .map_err(|e| format!("KDL parse error: {}", e))?
                    .nodes_mut().remove(0);
                ring_children.nodes_mut().push(node);
            }
            "border_status" | "border_width" | "border_active" | "border_inactive" => {
                let border_idx = if let Some(idx) = layout_children.nodes().iter().position(|n| n.name().value() == "border") {
                    idx
                } else {
                    let new_border = kdl::KdlNode::new("border");
                    layout_children.nodes_mut().push(new_border);
                    layout_children.nodes().len() - 1
                };
                let border = &mut layout_children.nodes_mut()[border_idx];
                let border_children = border.ensure_children();

                if id == "border_status" {
                    if let Some(pos) = border_children.nodes().iter().position(|n| n.name().value() == "off") {
                        border_children.nodes_mut().remove(pos);
                    }
                    if value.to_lowercase() == "off" {
                        let off_node = kdl::KdlNode::new("off");
                        border_children.nodes_mut().push(off_node);
                    }
                } else {
                    let prop_name = match id {
                        "border_width" => "width",
                        "border_active" => "active-color",
                        _ => "inactive-color",
                    };

                    if let Some(pos) = border_children.nodes().iter().position(|n| n.name().value() == prop_name) {
                        border_children.nodes_mut().remove(pos);
                    }

                    let formatted_value = if prop_name.contains("color") {
                        if !value.starts_with('"') {
                            format!("\"{}\"", value)
                        } else {
                            value
                        }
                    } else {
                        value
                    };

                    let snippet = format!("    {} {}\n", prop_name, formatted_value);
                    let node = snippet.parse::<KdlDocument>()
                        .map_err(|e| format!("KDL parse error: {}", e))?
                        .nodes_mut().remove(0);
                    border_children.nodes_mut().push(node);
                }
            }
            "corner_radius" => {
                let mut found_idx = None;
                for (idx, node) in doc.nodes().iter().enumerate() {
                    if node.name().value() == "window-rule" {
                        if let Some(children) = node.children() {
                            if children.nodes().iter().any(|n| n.name().value() == "geometry-corner-radius") {
                                found_idx = Some(idx);
                                break;
                            }
                        }
                    }
                }

                let rule_idx = if let Some(idx) = found_idx {
                    idx
                } else {
                    let mut new_rule = kdl::KdlNode::new("window-rule");
                    let rule_children = new_rule.ensure_children();
                    rule_children.nodes_mut().push(kdl::KdlNode::new("clip-to-geometry"));
                    
                    let draw_border_snippet = "draw-border-with-background false\n";
                    let draw_border_node = draw_border_snippet.parse::<KdlDocument>()
                        .unwrap()
                        .nodes_mut()
                        .remove(0);
                    rule_children.nodes_mut().push(draw_border_node);

                    doc.nodes_mut().push(new_rule);
                    doc.nodes().len() - 1
                };

                let rule = &mut doc.nodes_mut()[rule_idx];
                let rule_children = rule.ensure_children();

                if let Some(pos) = rule_children.nodes().iter().position(|n| n.name().value() == "geometry-corner-radius") {
                    rule_children.nodes_mut().remove(pos);
                }

                let snippet = format!("    geometry-corner-radius {}\n", value);
                let node = snippet.parse::<KdlDocument>()
                    .map_err(|e| format!("KDL parse error: {}", e))?
                    .nodes_mut().remove(0);
                rule_children.nodes_mut().push(node);
            }
            _ => {}
        }

        self.save_and_validate_changes()?;
        Ok(())
    }
}

pub fn get_action_desc(node: &kdl::KdlNode) -> String {
    if let Some(action_doc) = node.children() {
        let mut actions = Vec::new();
        for action_node in action_doc.nodes() {
            let act_name = action_node.name().value();
            let args: Vec<String> = action_node
                .entries()
                .iter()
                .map(|e| e.to_string())
                .collect();
            if args.is_empty() {
                actions.push(act_name.to_string());
            } else {
                actions.push(format!("{} {}", act_name, args.join(" ")));
            }
        }
        actions.join(", ")
    } else {
        let args: Vec<String> = node
            .entries()
            .iter()
            .map(|e| e.to_string())
            .collect();
        args.join(" ")
    }
}

pub fn get_default_keybindings() -> Vec<(String, String)> {
    let mut bindings = Vec::new();
    if let Ok(doc) = default_config::DEFAULT_CONFIG.parse::<KdlDocument>() {
        if let Some(binds_node) = doc.nodes().iter().find(|n| n.name().value() == "binds") {
            if let Some(children) = binds_node.children() {
                for node in children.nodes() {
                    let key = node.name().value();
                    let action_desc = get_action_desc(node);
                    bindings.push((key.to_string(), action_desc));
                }
            }
        }
    }
    bindings
}
