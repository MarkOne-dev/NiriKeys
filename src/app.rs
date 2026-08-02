use crate::default_config;
use crate::system::{validate_config, validate_noctalia_config};
use crate::translations::{Language, Translations};
use kdl::KdlDocument;
use ratatui::widgets::ListState;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Represents the active screen state of the TUI application.
#[derive(Clone, Debug)]
pub enum ActiveScreen {
    Loading {
        progress: u16,
        status_msg: String,
    },
    InstallPrompt {
        pm_name: String,
        cmd: String,
    },
    Dashboard,
    AddPopup,
    ConfirmOverwrite {
        key: String,
        action: String,
    },
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
    EditNoctaliaPopup {
        setting_id: String,
        setting_name: String,
        input_value: String,
        value_type: NoctaliaValueType,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoctaliaValueType {
    Bool,
    Float,
    Integer,
    Str,
}

/// Represents a configurable layout or aesthetic setting in Niri.
#[derive(Clone, Debug)]
pub struct AppearanceSetting {
    pub id: String,
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct NoctaliaSettingItem {
    pub id: String,
    pub name: String,
    pub value: String,
    pub value_type: NoctaliaValueType,
}

/// Represents the current field in focus inside the Add Shortcut popup.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InputFocus {
    Key,
    Action,
}

/// The main application state container.
pub struct App {
    pub config_path: PathBuf,
    pub dry_run: bool,
    pub lang: Language,
    pub doc: Option<KdlDocument>,
    pub keybindings: Vec<(String, String)>,
    pub list_state: ListState,
    pub active_screen: ActiveScreen,
    pub input_key: String,
    pub input_action: String,
    pub input_focus: InputFocus,
    pub file_size_kb: f64,
    pub file_mod_time: String,
    pub file_is_valid: bool,
    pub active_tab: usize,
    pub appearance_state: ListState,
    pub noctalia_path: PathBuf,
    pub noctalia_config: Option<toml::Table>,
    pub noctalia_settings: Vec<NoctaliaSettingItem>,
    pub noctalia_is_valid: bool,
    pub noctalia_state: ListState,
    pub agent_logs: Vec<String>,
    pub preview_scroll: u16,
}

impl App {
    /// Creates a new TUI App instance with the provided config path, dry-run flag, and language.
    pub fn new(config_path: PathBuf, dry_run: bool, lang: Language) -> Self {
        let mut noctalia_path = dirs::home_dir().unwrap_or_default();
        noctalia_path.push(".local/state/noctalia/settings.toml");

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
            noctalia_path,
            noctalia_config: None,
            noctalia_settings: Vec::new(),
            noctalia_is_valid: false,
            noctalia_state: ListState::default(),
            agent_logs: Vec::new(),
            preview_scroll: 0,
        }
    }

    /// Logs an activity event with a timestamp.
    pub fn log_agent_activity(&mut self, message: String) {
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        self.agent_logs.push(format!("[{}] {}", now, message));
        if self.agent_logs.len() > 100 {
            self.agent_logs.remove(0);
        }
    }

    /// Generates a live string preview of the active configuration file.
    pub fn get_live_file_preview(&self) -> String {
        if self.active_tab == 2 {
            if let Some(ref config) = self.noctalia_config {
                toml::to_string(config).unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            if let Some(ref doc) = self.doc {
                doc.to_string()
            } else {
                String::new()
            }
        }
    }

    /// Initializes the App state, loading the KDL document if the file exists.
    pub fn init(&mut self) -> Result<(), String> {
        self.log_agent_activity("Inicializando NiriKeys / Initializing NiriKeys...".to_string());
        if self.config_path.exists() {
            self.load_doc()?;
            self.update_metadata();
        }
        self.load_noctalia_config()?;
        Ok(())
    }

    /// Loads and parses the KDL configuration file, normalizing line endings.
    pub fn load_doc(&mut self) -> Result<(), String> {
        self.log_agent_activity(format!("Cargando Niri KDL / Loading Niri KDL from: {}", self.config_path.display()));
        let raw_content = fs::read_to_string(&self.config_path).map_err(|e| match self.lang {
            Language::Es => format!("Error al leer archivo: {}", e),
            Language::En => format!("Error reading file: {}", e),
        })?;
        let content = raw_content.replace("\r\n", "\n").replace('\r', "\n");

        let parsed_doc: KdlDocument = content.parse().map_err(|e| match self.lang {
            Language::Es => format!("Error al parsear KDL: {}", e),
            Language::En => format!("Error parsing KDL: {}", e),
        })?;

        self.doc = Some(parsed_doc);
        self.reload_keybindings()?;
        Ok(())
    }

    /// Reloads keybindings from the parsed KDL document.
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

    /// Updates the configuration file metadata attributes.
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

    /// Creates a default configuration file based on the embedded template.
    pub fn create_default_config(&mut self) -> Result<(), String> {
        let parent = self.config_path.parent().ok_or_else(|| match self.lang {
            Language::Es => "No se pudo detectar el directorio raíz".to_string(),
            Language::En => "Could not detect parent directory".to_string(),
        })?;

        fs::create_dir_all(parent).map_err(|e| match self.lang {
            Language::Es => format!("No se pudo crear directorio {}: {}", parent.display(), e),
            Language::En => format!("Could not create directory {}: {}", parent.display(), e),
        })?;

        fs::write(&self.config_path, default_config::DEFAULT_CONFIG).map_err(|e| {
            match self.lang {
                Language::Es => format!("Error al escribir archivo por defecto: {}", e),
                Language::En => format!("Error writing default file: {}", e),
            }
        })?;

        self.load_doc()?;
        self.update_metadata();
        self.active_screen = ActiveScreen::Dashboard;
        Ok(())
    }

    /// Moves keybinding list selection up.
    pub fn move_selection_up(&mut self) {
        if self.keybindings.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = if current == 0 {
            self.keybindings.len() - 1
        } else {
            current - 1
        };
        self.list_state.select(Some(next));
    }

    /// Moves keybinding list selection down.
    pub fn move_selection_down(&mut self) {
        if self.keybindings.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = if current >= self.keybindings.len() - 1 {
            0
        } else {
            current + 1
        };
        self.list_state.select(Some(next));
    }

    /// Prepares inputs and opens the Add Shortcut dialog.
    pub fn enter_add_mode(&mut self) {
        self.input_key.clear();
        self.input_action.clear();
        self.input_focus = InputFocus::Key;
        self.active_screen = ActiveScreen::AddPopup;
    }

    /// Deletes the currently selected keybinding from the configuration.
    pub fn delete_selected(&mut self) {
        let selected = match self.list_state.selected() {
            Some(idx) => idx,
            Option::None => return,
        };

        let (key, _) = &self.keybindings[selected];
        let key_to_remove = key.clone();
        self.log_agent_activity(format!("Eliminando atajo / Deleting keybinding: {}", key_to_remove));

        let doc = match self.doc.as_mut() {
            Some(d) => d,
            Option::None => return,
        };

        if let Some(binds_node) = doc
            .nodes_mut()
            .iter_mut()
            .find(|n| n.name().value() == "binds")
        {
            let children = binds_node.ensure_children();
            if let Some(pos) = children
                .nodes()
                .iter()
                .position(|n| n.name().value() == key_to_remove)
            {
                children.nodes_mut().remove(pos);
            }
        }

        if let Err(e) = self.save_and_validate_changes() {
            self.active_screen = ActiveScreen::ErrorPopup(e);
        } else {
            self.active_screen = ActiveScreen::InfoPopup(
                Translations::get(&self.lang)
                    .msg_deleted_success
                    .to_string(),
            );
        }
    }

    /// Creates a backup copy of the configuration file on disk.
    pub fn trigger_backup(&mut self) {
        self.log_agent_activity("Creando copia de seguridad / Creating backup copy...".to_string());
        if !self.config_path.exists() {
            self.active_screen = ActiveScreen::ErrorPopup(match self.lang {
                Language::Es => "No existe archivo de configuración".to_string(),
                Language::En => "Configuration file does not exist".to_string(),
            });
            return;
        }

        let mut backup_path = self.config_path.clone();
        let ext = backup_path
            .extension()
            .map(|e| format!("{}.bak", e.to_string_lossy()))
            .unwrap_or_else(|| "bak".to_string());
        backup_path.set_extension(ext);

        match fs::copy(&self.config_path, &backup_path) {
            Ok(_) => {
                self.log_agent_activity(format!("Copia de seguridad creada con éxito / Backup created successfully at: {}", backup_path.display()));
                self.active_screen = ActiveScreen::InfoPopup(format!(
                    "{}",
                    Translations::get(&self.lang)
                        .msg_backup_success
                        .replace("{}", &backup_path.to_string_lossy())
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

    /// Toggles the focus between key combination and action input fields.
    pub fn toggle_input_focus(&mut self) {
        self.input_focus = match self.input_focus {
            InputFocus::Key => InputFocus::Action,
            InputFocus::Action => InputFocus::Key,
        };
    }

    /// Removes the last character from the active input field.
    pub fn handle_backspace(&mut self) {
        match self.input_focus {
            InputFocus::Key => {
                self.input_key.pop();
            }
            InputFocus::Action => {
                self.input_action.pop();
            }
        }
    }

    /// Pushes a character to the active input field.
    pub fn handle_char(&mut self, c: char) {
        match self.input_focus {
            InputFocus::Key => {
                self.input_key.push(c);
            }
            InputFocus::Action => {
                self.input_action.push(c);
            }
        }
    }

    /// Processes the submission of the Add Shortcut form.
    pub fn submit_add_form(&mut self) {
        let key = self.input_key.trim().to_string();
        let action = self.input_action.trim().to_string();

        if key.is_empty() || action.is_empty() {
            self.active_screen = ActiveScreen::ErrorPopup(
                Translations::get(&self.lang).msg_empty_fields.to_string(),
            );
            return;
        }

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

    /// Applies a keybinding to the configuration, looking it up in the template if possible.
    pub fn apply_keybinding(&mut self, key: String, action: String) {
        self.log_agent_activity(format!("Aplicando atajo / Applying keybinding: {} -> {}", key, action));
        let doc = match self.doc.as_mut() {
            Some(d) => d,
            Option::None => return,
        };

        let binds_node = if let Some(idx) = doc
            .nodes()
            .iter()
            .position(|node| node.name().value() == "binds")
        {
            &mut doc.nodes_mut()[idx]
        } else {
            let new_binds = kdl::KdlNode::new("binds");
            doc.nodes_mut().push(new_binds);
            let len = doc.nodes().len();
            &mut doc.nodes_mut()[len - 1]
        };

        let children = binds_node.ensure_children();

        if let Some(pos) = children
            .nodes()
            .iter()
            .position(|n| n.name().value() == key)
        {
            children.nodes_mut().remove(pos);
        }

        let default_doc = default_config::DEFAULT_CONFIG.parse::<KdlDocument>().ok();
        let mut node_to_add = None;
        if let Some(ref def_doc) = default_doc {
            if let Some(def_binds) = def_doc.nodes().iter().find(|n| n.name().value() == "binds") {
                if let Some(def_children) = def_binds.children() {
                    if let Some(found_node) = def_children
                        .nodes()
                        .iter()
                        .find(|n| n.name().value() == key)
                    {
                        node_to_add = Some(found_node.clone());
                    }
                }
            }
        }

        let parsed_snippet = if let Some(mut node) = node_to_add {
            if let Some(fmt) = node.format_mut() {
                fmt.leading = "    ".to_string();
                if !fmt.terminator.ends_with('\n') {
                    fmt.terminator = "\n".to_string();
                }
            }
            node
        } else {
            let formatted_action = if Self::is_niri_native_action(&action) {
                action
            } else if action.contains(' ') && !action.starts_with('"') {
                let parts: Vec<&str> = action.split_whitespace().collect();
                let spawn_args = parts
                    .iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("spawn {}", spawn_args)
            } else if !action.starts_with('"') {
                format!("spawn \"{}\"", action)
            } else {
                action
            };

            let snippet = format!("    \"{}\" {{ {}; }}\n", key, formatted_action);
            match snippet.parse::<KdlDocument>() {
                Ok(mut temp_doc) => temp_doc.nodes_mut().remove(0),
                Err(e) => {
                    self.active_screen = ActiveScreen::ErrorPopup(match self.lang {
                        Language::Es => format!("KDL inválido: {}", e),
                        Language::En => format!("Invalid KDL: {}", e),
                    });
                    return;
                }
            }
        };

        children.nodes_mut().push(parsed_snippet);

        if let Err(e) = self.save_and_validate_changes() {
            self.active_screen = ActiveScreen::ErrorPopup(e);
        } else {
            self.active_screen =
                ActiveScreen::InfoPopup(Translations::get(&self.lang).msg_save_success.to_string());
        }
    }

    /// Applies a batch of keybindings, checking against the default template.
    pub fn apply_keybindings_batch(
        &mut self,
        new_bindings: Vec<(String, String)>,
    ) -> Result<(), String> {
        let doc = match self.doc.as_mut() {
            Some(d) => d,
            Option::None => return Err("KDL document not loaded".to_string()),
        };

        let default_doc = default_config::DEFAULT_CONFIG.parse::<KdlDocument>().ok();

        let binds_node = if let Some(idx) = doc
            .nodes()
            .iter()
            .position(|node| node.name().value() == "binds")
        {
            &mut doc.nodes_mut()[idx]
        } else {
            let new_binds = kdl::KdlNode::new("binds");
            doc.nodes_mut().push(new_binds);
            let len = doc.nodes().len();
            &mut doc.nodes_mut()[len - 1]
        };

        let children = binds_node.ensure_children();

        for (key, action) in new_bindings {
            if let Some(pos) = children
                .nodes()
                .iter()
                .position(|n| n.name().value() == key)
            {
                children.nodes_mut().remove(pos);
            }

            let mut node_to_add = None;
            if let Some(ref def_doc) = default_doc {
                if let Some(def_binds) =
                    def_doc.nodes().iter().find(|n| n.name().value() == "binds")
                {
                    if let Some(def_children) = def_binds.children() {
                        if let Some(found_node) = def_children
                            .nodes()
                            .iter()
                            .find(|n| n.name().value() == key)
                        {
                            node_to_add = Some(found_node.clone());
                        }
                    }
                }
            }

            let parsed_node = if let Some(mut node) = node_to_add {
                if let Some(fmt) = node.format_mut() {
                    fmt.leading = "    ".to_string();
                    if !fmt.terminator.ends_with('\n') {
                        fmt.terminator = "\n".to_string();
                    }
                }
                node
            } else {
                let formatted_action = if Self::is_niri_native_action(&action) {
                    action
                } else if action.contains(' ') && !action.starts_with('"') {
                    let parts: Vec<&str> = action.split_whitespace().collect();
                    let spawn_args = parts
                        .iter()
                        .map(|p| format!("\"{}\"", p))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("spawn {}", spawn_args)
                } else if !action.starts_with('"') {
                    format!("spawn \"{}\"", action)
                } else {
                    action
                };

                let snippet = format!("    \"{}\" {{ {}; }}\n", key, formatted_action);
                snippet
                    .parse::<KdlDocument>()
                    .map_err(|e| {
                        format!("KDL snippet parse error: {} for snippet: {}", e, snippet)
                    })?
                    .nodes_mut()
                    .remove(0)
            };

            children.nodes_mut().push(parsed_node);
        }

        self.save_and_validate_changes()?;
        Ok(())
    }

    /// Saves the current configuration to disk and validates it using Niri.
    pub fn save_and_validate_changes(&mut self) -> Result<(), String> {
        self.log_agent_activity("Validando cambios en Niri config / Validating Niri config changes...".to_string());
        let doc = self.doc.as_mut().ok_or_else(|| match self.lang {
            Language::Es => "No hay ningún documento cargado".to_string(),
            Language::En => "No document loaded".to_string(),
        })?;
        Self::fix_trailing_comments(doc);
        let serialized = doc.to_string();

        if self.dry_run {
            self.log_agent_activity("Simulación activa (dry-run). No se escriben cambios en disco. / Dry-run active. No changes written.".to_string());
            self.reload_keybindings()?;
            self.update_metadata();
            return Ok(());
        }

        let parent = self.config_path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|e| match self.lang {
            Language::Es => format!("No se pudo crear directorio parent: {}", e),
            Language::En => format!("Could not create parent directory: {}", e),
        })?;

        let mut temp_file = NamedTempFile::new_in(parent).map_err(|e| match self.lang {
            Language::Es => format!("No se pudo crear archivo temporal: {}", e),
            Language::En => format!("Could not create temporary file: {}", e),
        })?;

        use std::io::Write;
        temp_file
            .write_all(serialized.as_bytes())
            .map_err(|e| match self.lang {
                Language::Es => format!("Error al escribir archivo temporal: {}", e),
                Language::En => format!("Error writing temporary file: {}", e),
            })?;

        match validate_config(temp_file.path()) {
            Ok(_) => {
                self.log_agent_activity("Validación de Niri exitosa. Persistiendo cambios / Niri validation OK. Persisting changes...".to_string());
                temp_file
                    .persist(&self.config_path)
                    .map_err(|e| match self.lang {
                        Language::Es => format!("Error al guardar el archivo definitivo: {}", e),
                        Language::En => format!("Error saving final configuration file: {}", e),
                    })?;

                self.load_doc()?;
                self.update_metadata();
                Ok(())
            }
            Err(err_msg) => {
                self.log_agent_activity(format!("¡ERROR de validación de Niri! / Niri validation ERROR: {}", err_msg));
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

    /// Fetches all configurable appearance options and values.
    pub fn get_appearance_settings(&self) -> Vec<AppearanceSetting> {
        let mut settings = Vec::new();

        let doc = match &self.doc {
            Some(d) => d,
            Option::None => return settings,
        };

        let layout_node = doc.nodes().iter().find(|n| n.name().value() == "layout");

        let get_layout_val = |child_name: &str| -> String {
            if let Some(layout) = layout_node {
                if let Some(children) = layout.children() {
                    if let Some(child) = children
                        .nodes()
                        .iter()
                        .find(|n| n.name().value() == child_name)
                    {
                        if let Some(val) = child.entries().first() {
                            return val.value().to_string();
                        }
                    }
                }
            }
            "Default".to_string()
        };

        let get_subnode_val = |node_name: &str, child_name: &str| -> String {
            if let Some(layout) = layout_node {
                if let Some(children) = layout.children() {
                    if let Some(sub) = children
                        .nodes()
                        .iter()
                        .find(|n| n.name().value() == node_name)
                    {
                        if let Some(sub_children) = sub.children() {
                            if let Some(child) = sub_children
                                .nodes()
                                .iter()
                                .find(|n| n.name().value() == child_name)
                            {
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

        let is_border_off = || -> bool {
            if let Some(layout) = layout_node {
                if let Some(children) = layout.children() {
                    if let Some(sub) = children
                        .nodes()
                        .iter()
                        .find(|n| n.name().value() == "border")
                    {
                        if let Some(sub_children) = sub.children() {
                            return sub_children
                                .nodes()
                                .iter()
                                .any(|n| n.name().value() == "off");
                        }
                    }
                }
            }
            true
        };

        let get_corner_radius = || -> String {
            for node in doc.nodes() {
                if node.name().value() == "window-rule" {
                    if let Some(children) = node.children() {
                        if let Some(child) = children
                            .nodes()
                            .iter()
                            .find(|n| n.name().value() == "geometry-corner-radius")
                        {
                            if let Some(val) = child.entries().first() {
                                return val.value().to_string();
                            }
                        }
                    }
                }
            }
            "Default".to_string()
        };

        settings.push(AppearanceSetting {
            id: "gaps".to_string(),
            name: match self.lang {
                Language::Es => "Espaciado general (Gaps)".to_string(),
                Language::En => "General Gaps".to_string(),
            },
            value: get_layout_val("gaps"),
        });

        settings.push(AppearanceSetting {
            id: "focus_ring_width".to_string(),
            name: match self.lang {
                Language::Es => "Foco: Grosor del anillo".to_string(),
                Language::En => "Focus Ring: Width".to_string(),
            },
            value: get_subnode_val("focus-ring", "width"),
        });

        settings.push(AppearanceSetting {
            id: "focus_ring_active".to_string(),
            name: match self.lang {
                Language::Es => "Foco: Color activo".to_string(),
                Language::En => "Focus Ring: Active Color".to_string(),
            },
            value: get_subnode_val("focus-ring", "active-color"),
        });

        settings.push(AppearanceSetting {
            id: "focus_ring_inactive".to_string(),
            name: match self.lang {
                Language::Es => "Foco: Color inactivo".to_string(),
                Language::En => "Focus Ring: Inactive Color".to_string(),
            },
            value: get_subnode_val("focus-ring", "inactive-color"),
        });

        settings.push(AppearanceSetting {
            id: "border_status".to_string(),
            name: match self.lang {
                Language::Es => "Bordes de ventana (on/off)".to_string(),
                Language::En => "Window Borders (on/off)".to_string(),
            },
            value: if is_border_off() {
                "off".to_string()
            } else {
                "on".to_string()
            },
        });

        settings.push(AppearanceSetting {
            id: "border_width".to_string(),
            name: match self.lang {
                Language::Es => "Borde: Grosor".to_string(),
                Language::En => "Border: Width".to_string(),
            },
            value: get_subnode_val("border", "width"),
        });

        settings.push(AppearanceSetting {
            id: "border_active".to_string(),
            name: match self.lang {
                Language::Es => "Borde: Color activo".to_string(),
                Language::En => "Border: Active Color".to_string(),
            },
            value: get_subnode_val("border", "active-color"),
        });

        settings.push(AppearanceSetting {
            id: "border_inactive".to_string(),
            name: match self.lang {
                Language::Es => "Borde: Color inactivo".to_string(),
                Language::En => "Border: Inactive Color".to_string(),
            },
            value: get_subnode_val("border", "inactive-color"),
        });

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

    /// Updates the specified appearance option to a new value in the config file.
    pub fn update_appearance_setting(&mut self, id: &str, value: String) -> Result<(), String> {
        self.log_agent_activity(format!("Modificando apariencia / Modifying appearance: {} -> {}", id, value));
        let doc = match self.doc.as_mut() {
            Some(d) => d,
            Option::None => return Err("KDL document not loaded".to_string()),
        };

        let layout_idx = if let Some(idx) = doc
            .nodes()
            .iter()
            .position(|n| n.name().value() == "layout")
        {
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
                if let Some(pos) = layout_children
                    .nodes()
                    .iter()
                    .position(|n| n.name().value() == "gaps")
                {
                    layout_children.nodes_mut().remove(pos);
                }
                let snippet = format!("    gaps {}\n", value);
                let node = snippet
                    .parse::<KdlDocument>()
                    .map_err(|e| format!("KDL parse error: {}", e))?
                    .nodes_mut()
                    .remove(0);
                layout_children.nodes_mut().push(node);
            }
            "focus_ring_width" | "focus_ring_active" | "focus_ring_inactive" => {
                let ring_idx = if let Some(idx) = layout_children
                    .nodes()
                    .iter()
                    .position(|n| n.name().value() == "focus-ring")
                {
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

                if let Some(pos) = ring_children
                    .nodes()
                    .iter()
                    .position(|n| n.name().value() == prop_name)
                {
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
                let node = snippet
                    .parse::<KdlDocument>()
                    .map_err(|e| format!("KDL parse error: {}", e))?
                    .nodes_mut()
                    .remove(0);
                ring_children.nodes_mut().push(node);
            }
            "border_status" | "border_width" | "border_active" | "border_inactive" => {
                let border_idx = if let Some(idx) = layout_children
                    .nodes()
                    .iter()
                    .position(|n| n.name().value() == "border")
                {
                    idx
                } else {
                    let new_border = kdl::KdlNode::new("border");
                    layout_children.nodes_mut().push(new_border);
                    layout_children.nodes().len() - 1
                };
                let border = &mut layout_children.nodes_mut()[border_idx];
                let border_children = border.ensure_children();

                if id == "border_status" {
                    if let Some(pos) = border_children
                        .nodes()
                        .iter()
                        .position(|n| n.name().value() == "off")
                    {
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

                    if let Some(pos) = border_children
                        .nodes()
                        .iter()
                        .position(|n| n.name().value() == prop_name)
                    {
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
                    let node = snippet
                        .parse::<KdlDocument>()
                        .map_err(|e| format!("KDL parse error: {}", e))?
                        .nodes_mut()
                        .remove(0);
                    border_children.nodes_mut().push(node);
                }
            }
            "corner_radius" => {
                let mut found_idx = None;
                for (idx, node) in doc.nodes().iter().enumerate() {
                    if node.name().value() == "window-rule" {
                        if let Some(children) = node.children() {
                            if children
                                .nodes()
                                .iter()
                                .any(|n| n.name().value() == "geometry-corner-radius")
                            {
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
                    rule_children
                        .nodes_mut()
                        .push(kdl::KdlNode::new("clip-to-geometry"));

                    let draw_border_snippet = "draw-border-with-background false\n";
                    let draw_border_node = draw_border_snippet
                        .parse::<KdlDocument>()
                        .unwrap()
                        .nodes_mut()
                        .remove(0);
                    rule_children.nodes_mut().push(draw_border_node);

                    doc.nodes_mut().push(new_rule);
                    doc.nodes().len() - 1
                };

                let rule = &mut doc.nodes_mut()[rule_idx];
                let rule_children = rule.ensure_children();

                if let Some(pos) = rule_children
                    .nodes()
                    .iter()
                    .position(|n| n.name().value() == "geometry-corner-radius")
                {
                    rule_children.nodes_mut().remove(pos);
                }

                let snippet = format!("    geometry-corner-radius {}\n", value);
                let node = snippet
                    .parse::<KdlDocument>()
                    .map_err(|e| format!("KDL parse error: {}", e))?
                    .nodes_mut()
                    .remove(0);
                rule_children.nodes_mut().push(node);
            }
            _ => {}
        }

        self.save_and_validate_changes()?;
        Ok(())
    }

    fn fix_trailing_comments(doc: &mut kdl::KdlDocument) {
        for node in doc.nodes_mut() {
            Self::fix_node_comments(node);
        }
    }

    fn fix_node_comments(node: &mut kdl::KdlNode) {
        if let Some(format) = node.format_mut() {
            let trimmed = format.trailing.trim_start();
            if trimmed.starts_with("//") && !format.trailing.ends_with('\n') {
                format.trailing.push('\n');
            }
        }

        if let Some(children) = node.children_mut() {
            Self::fix_trailing_comments(children);
        }
    }

    /// Loads and parses the Noctalia TOML configuration file if it exists.
    pub fn load_noctalia_config(&mut self) -> Result<(), String> {
        if self.noctalia_path.exists() {
            self.log_agent_activity(format!("Cargando Noctalia TOML / Loading Noctalia TOML from: {}", self.noctalia_path.display()));
            let content = fs::read_to_string(&self.noctalia_path).map_err(|e| match self.lang {
                Language::Es => format!("Error al leer configuración de Noctalia: {}", e),
                Language::En => format!("Error reading Noctalia configuration: {}", e),
            })?;
            
            let config: toml::Table = toml::from_str(&content).map_err(|e| match self.lang {
                Language::Es => format!("Error al parsear TOML de Noctalia: {}", e),
                Language::En => format!("Error parsing Noctalia TOML: {}", e),
            })?;
            
            self.noctalia_config = Some(config);
            self.noctalia_is_valid = true;
            self.reload_noctalia_settings();
        } else {
            self.noctalia_config = None;
            self.noctalia_is_valid = false;
            self.noctalia_settings.clear();
        }
        Ok(())
    }

    /// Saves the current Noctalia TOML configuration back to the disk.
    pub fn save_noctalia_config(&mut self) -> Result<(), String> {
        self.log_agent_activity("Validando cambios de Noctalia / Validating Noctalia changes...".to_string());
        if self.dry_run {
            self.log_agent_activity("Simulación activa (dry-run). No se guardan cambios de Noctalia. / Dry-run active. Noctalia changes not saved.".to_string());
            return Ok(());
        }
        if let Some(ref config) = self.noctalia_config {
            let content = toml::to_string(config).map_err(|e| match self.lang {
                Language::Es => format!("Error al serializar TOML de Noctalia: {}", e),
                Language::En => format!("Error serializing Noctalia TOML: {}", e),
            })?;
            
            let parent = self.noctalia_path.parent().unwrap_or_else(|| Path::new("."));
            fs::create_dir_all(parent).map_err(|e| match self.lang {
                Language::Es => format!("Error al crear directorio para Noctalia: {}", e),
                Language::En => format!("Error creating Noctalia directory: {}", e),
            })?;
            
            let mut temp_file = NamedTempFile::new_in(parent).map_err(|e| match self.lang {
                Language::Es => format!("No se pudo crear archivo temporal: {}", e),
                Language::En => format!("Could not create temporary file: {}", e),
            })?;
            
            use std::io::Write;
            temp_file
                .write_all(content.as_bytes())
                .map_err(|e| match self.lang {
                    Language::Es => format!("Error al escribir archivo temporal: {}", e),
                    Language::En => format!("Error writing temporary file: {}", e),
                })?;
                
            match validate_noctalia_config(temp_file.path()) {
                Ok(_) => {
                    self.log_agent_activity("Validación de Noctalia exitosa. Guardando cambios... / Noctalia validation OK. Saving changes...".to_string());
                    temp_file
                        .persist(&self.noctalia_path)
                        .map_err(|e| match self.lang {
                            Language::Es => format!("Error al guardar el archivo definitivo de Noctalia: {}", e),
                            Language::En => format!("Error saving final Noctalia configuration file: {}", e),
                        })?;
                    
                    // Trigger hot-reload in Noctalia
                    self.log_agent_activity("Enviando señal de recarga / Sending config hot-reload command: 'noctalia msg config-reload'".to_string());
                    let reload_output = std::process::Command::new("noctalia")
                        .arg("msg")
                        .arg("config-reload")
                        .output();
                    match reload_output {
                        Ok(out) => {
                            if out.status.success() {
                                self.log_agent_activity("¡Recarga de Noctalia exitosa! / Noctalia hot-reload successful!".to_string());
                            } else {
                                let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                                self.log_agent_activity(format!("Error en comando de recarga / Hot-reload failed: {}", err));
                            }
                        }
                        Err(e) => {
                            self.log_agent_activity(format!("No se pudo ejecutar comando de recarga / Failed to run hot-reload command: {}", e));
                        }
                    }
                    Ok(())
                }
                Err(validation_error) => {
                    self.log_agent_activity(format!("¡ERROR de validación de Noctalia! / Noctalia validation ERROR: {}", validation_error));
                    Err(validation_error)
                }
            }
        } else {
            Ok(())
        }
    }
      /// Refreshes the list of Noctalia UI settings exposed in the TUI menu.
    pub fn reload_noctalia_settings(&mut self) {
        let mut settings = Vec::new();
        let config = match &self.noctalia_config {
            Some(c) => c,
            _ => {
                self.noctalia_settings.clear();
                return;
            }
        };

        let get_val = |path: &str| -> Option<toml::Value> {
            let mut current = toml::Value::Table(config.clone());
            for key in path.split('.') {
                current = current.get(key)?.clone();
            }
            Some(current)
        };

        let get_bool = |path: &str| -> String {
            get_val(path)
                .and_then(|v| v.as_bool())
                .map(|b| b.to_string())
                .unwrap_or_else(|| "Default".to_string())
        };

        let get_float = |path: &str| -> String {
            get_val(path)
                .and_then(|v| v.as_float())
                .map(|f| format!("{:.2}", f))
                .unwrap_or_else(|| "Default".to_string())
        };

        let get_int = |path: &str| -> String {
            get_val(path)
                .and_then(|v| v.as_integer())
                .map(|i| i.to_string())
                .unwrap_or_else(|| "Default".to_string())
        };

        let get_str = |path: &str| -> String {
            get_val(path)
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "Default".to_string())
        };

        // --- BACKDROP ---
        settings.push(NoctaliaSettingItem {
            id: "backdrop.enabled".to_string(),
            name: match self.lang {
                Language::Es => "Fondo: Difuminado de fondo (on/off)".to_string(),
                Language::En => "Backdrop: Blurred Wallpaper (on/off)".to_string(),
            },
            value: get_bool("backdrop.enabled"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "backdrop.blur_intensity".to_string(),
            name: match self.lang {
                Language::Es => "Fondo: Intensidad de desenfoque".to_string(),
                Language::En => "Backdrop: Blur Intensity (0.0 - 1.0)".to_string(),
            },
            value: get_float("backdrop.blur_intensity"),
            value_type: NoctaliaValueType::Float,
        });

        settings.push(NoctaliaSettingItem {
            id: "backdrop.tint_intensity".to_string(),
            name: match self.lang {
                Language::Es => "Fondo: Intensidad de tinte".to_string(),
                Language::En => "Backdrop: Tint Intensity (0.0 - 1.0)".to_string(),
            },
            value: get_float("backdrop.tint_intensity"),
            value_type: NoctaliaValueType::Float,
        });

        // --- SHELL GENERAL ---
        settings.push(NoctaliaSettingItem {
            id: "shell.niri_overview_type_to_launch_enabled".to_string(),
            name: match self.lang {
                Language::Es => "Overview: Escribir para lanzar (on/off)".to_string(),
                Language::En => "Overview: Type to Launch (on/off)".to_string(),
            },
            value: get_bool("shell.niri_overview_type_to_launch_enabled"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "shell.corner_radius_scale".to_string(),
            name: match self.lang {
                Language::Es => "Bordes: Escala de radio de esquina".to_string(),
                Language::En => "Borders: Corner Radius Scale".to_string(),
            },
            value: get_float("shell.corner_radius_scale"),
            value_type: NoctaliaValueType::Float,
        });

        settings.push(NoctaliaSettingItem {
            id: "shell.button_borders".to_string(),
            name: match self.lang {
                Language::Es => "Bordes: Bordes de botones (on/off)".to_string(),
                Language::En => "Borders: Button Borders (on/off)".to_string(),
            },
            value: get_bool("shell.button_borders"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "shell.input_borders".to_string(),
            name: match self.lang {
                Language::Es => "Bordes: Bordes de entradas (on/off)".to_string(),
                Language::En => "Borders: Input Borders (on/off)".to_string(),
            },
            value: get_bool("shell.input_borders"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "shell.popup_borders".to_string(),
            name: match self.lang {
                Language::Es => "Bordes: Bordes de menús popup (on/off)".to_string(),
                Language::En => "Borders: Popup Borders (on/off)".to_string(),
            },
            value: get_bool("shell.popup_borders"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "shell.card_borders".to_string(),
            name: match self.lang {
                Language::Es => "Bordes: Bordes de tarjetas/secciones (on/off)".to_string(),
                Language::En => "Borders: Card Borders (on/off)".to_string(),
            },
            value: get_bool("shell.card_borders"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "shell.popup_shadows".to_string(),
            name: match self.lang {
                Language::Es => "Sombras: Sombras en popups (on/off)".to_string(),
                Language::En => "Shadows: Popup Shadows (on/off)".to_string(),
            },
            value: get_bool("shell.popup_shadows"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "shell.font_family".to_string(),
            name: match self.lang {
                Language::Es => "Fuente: Familia tipográfica principal".to_string(),
                Language::En => "Font: Primary Family".to_string(),
            },
            value: get_str("shell.font_family"),
            value_type: NoctaliaValueType::Str,
        });

        settings.push(NoctaliaSettingItem {
            id: "shell.offline_mode".to_string(),
            name: match self.lang {
                Language::Es => "Red: Modo Fuera de Línea (on/off)".to_string(),
                Language::En => "Network: Offline Mode (on/off)".to_string(),
            },
            value: get_bool("shell.offline_mode"),
            value_type: NoctaliaValueType::Bool,
        });

        // --- ANIMATIONS ---
        settings.push(NoctaliaSettingItem {
            id: "shell.animation.enabled".to_string(),
            name: match self.lang {
                Language::Es => "Animación: Habilitar transiciones (on/off)".to_string(),
                Language::En => "Animation: Enable transitions (on/off)".to_string(),
            },
            value: get_bool("shell.animation.enabled"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "shell.animation.speed".to_string(),
            name: match self.lang {
                Language::Es => "Animación: Multiplicador de velocidad".to_string(),
                Language::En => "Animation: Speed Multiplier".to_string(),
            },
            value: get_float("shell.animation.speed"),
            value_type: NoctaliaValueType::Float,
        });

        // --- DOCK ---
        settings.push(NoctaliaSettingItem {
            id: "dock.enabled".to_string(),
            name: match self.lang {
                Language::Es => "Dock: Mostrar Dock (on/off)".to_string(),
                Language::En => "Dock: Show Dock (on/off)".to_string(),
            },
            value: get_bool("dock.enabled"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "dock.position".to_string(),
            name: match self.lang {
                Language::Es => "Dock: Posición (left/right/top/bottom)".to_string(),
                Language::En => "Dock: Position (left/right/top/bottom)".to_string(),
            },
            value: get_str("dock.position"),
            value_type: NoctaliaValueType::Str,
        });

        settings.push(NoctaliaSettingItem {
            id: "dock.icon_size".to_string(),
            name: match self.lang {
                Language::Es => "Dock: Tamaño de iconos (píxeles)".to_string(),
                Language::En => "Dock: Icon Size (pixels)".to_string(),
            },
            value: get_int("dock.icon_size"),
            value_type: NoctaliaValueType::Integer,
        });

        settings.push(NoctaliaSettingItem {
            id: "dock.background_opacity".to_string(),
            name: match self.lang {
                Language::Es => "Dock: Opacidad de fondo".to_string(),
                Language::En => "Dock: Background Opacity (0.0 - 1.0)".to_string(),
            },
            value: get_float("dock.background_opacity"),
            value_type: NoctaliaValueType::Float,
        });

        // --- THEME ---
        settings.push(NoctaliaSettingItem {
            id: "theme.mode".to_string(),
            name: match self.lang {
                Language::Es => "Tema: Modo (dark/light/auto)".to_string(),
                Language::En => "Theme: Mode (dark/light/auto)".to_string(),
            },
            value: get_str("theme.mode"),
            value_type: NoctaliaValueType::Str,
        });

        // --- ACCESSIBILITY ---
        settings.push(NoctaliaSettingItem {
            id: "accessibility.ui_scale".to_string(),
            name: match self.lang {
                Language::Es => "Accesibilidad: Escala global de la UI".to_string(),
                Language::En => "Accessibility: UI Scale".to_string(),
            },
            value: get_float("accessibility.ui_scale"),
            value_type: NoctaliaValueType::Float,
        });

        settings.push(NoctaliaSettingItem {
            id: "accessibility.high_contrast".to_string(),
            name: match self.lang {
                Language::Es => "Accesibilidad: Alto contraste (on/off)".to_string(),
                Language::En => "Accessibility: High Contrast (on/off)".to_string(),
            },
            value: get_bool("accessibility.high_contrast"),
            value_type: NoctaliaValueType::Bool,
        });

        // --- OSD ---
        settings.push(NoctaliaSettingItem {
            id: "osd.border".to_string(),
            name: match self.lang {
                Language::Es => "OSD: Bordes de popups (on/off)".to_string(),
                Language::En => "OSD: Outline on cards (on/off)".to_string(),
            },
            value: get_bool("osd.border"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "osd.scale".to_string(),
            name: match self.lang {
                Language::Es => "OSD: Multiplicador de escala de OSD".to_string(),
                Language::En => "OSD: Scale multiplier".to_string(),
            },
            value: get_float("osd.scale"),
            value_type: NoctaliaValueType::Float,
        });

        // --- LOCKSCREEN ---
        settings.push(NoctaliaSettingItem {
            id: "lockscreen.enabled".to_string(),
            name: match self.lang {
                Language::Es => "Bloqueo: Habilitar pantalla de bloqueo (on/off)".to_string(),
                Language::En => "Lockscreen: Enable session lock (on/off)".to_string(),
            },
            value: get_bool("lockscreen.enabled"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "lockscreen.fingerprint".to_string(),
            name: match self.lang {
                Language::Es => "Bloqueo: Permitir huella dactilar (on/off)".to_string(),
                Language::En => "Lockscreen: Allow fingerprint auth (on/off)".to_string(),
            },
            value: get_bool("lockscreen.fingerprint"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "lockscreen.blurred_desktop".to_string(),
            name: match self.lang {
                Language::Es => "Bloqueo: Fondo de pantalla capturado (on/off)".to_string(),
                Language::En => "Lockscreen: Blurred desktop capture (on/off)".to_string(),
            },
            value: get_bool("lockscreen.blurred_desktop"),
            value_type: NoctaliaValueType::Bool,
        });

        settings.push(NoctaliaSettingItem {
            id: "lockscreen.blur_intensity".to_string(),
            name: match self.lang {
                Language::Es => "Bloqueo: Intensidad de desenfoque de fondo".to_string(),
                Language::En => "Lockscreen: Blur Intensity (0.0 - 1.0)".to_string(),
            },
            value: get_float("lockscreen.blur_intensity"),
            value_type: NoctaliaValueType::Float,
        });

        settings.push(NoctaliaSettingItem {
            id: "lockscreen.tint_intensity".to_string(),
            name: match self.lang {
                Language::Es => "Bloqueo: Intensidad de tinte sobre fondo".to_string(),
                Language::En => "Lockscreen: Tint Intensity (0.0 - 1.0)".to_string(),
            },
            value: get_float("lockscreen.tint_intensity"),
            value_type: NoctaliaValueType::Float,
        });

        self.noctalia_settings = settings;

        let len = self.noctalia_settings.len();
        if len == 0 {
            self.noctalia_state.select(None);
        } else {
            let curr = self.noctalia_state.selected().unwrap_or(0);
            if curr >= len {
                self.noctalia_state.select(Some(len - 1));
            } else {
                self.noctalia_state.select(Some(curr));
            }
        }
    }

    /// Updates a specific Noctalia setting value and saves the configuration.
    pub fn update_noctalia_setting(&mut self, id: &str, value: String) -> Result<(), String> {
        self.log_agent_activity(format!("Modificando ajuste de Noctalia / Modifying Noctalia setting: {} -> {}", id, value));
        let config = match self.noctalia_config.as_mut() {
            Some(c) => c,
            _ => return Err(match self.lang {
                Language::Es => "Configuración de Noctalia no cargada".to_string(),
                Language::En => "Noctalia configuration not loaded".to_string(),
            }),
        };

        let mut update_path = |path: &str, val: toml::Value| {
            let parts: Vec<&str> = path.split('.').collect();
            match parts.len() {
                1 => {
                    config.insert(parts[0].to_string(), val);
                }
                2 => {
                    let sec = parts[0];
                    let key = parts[1];
                    if let Some(sec_val) = config.get_mut(sec) {
                        if let Some(sec_table) = sec_val.as_table_mut() {
                            sec_table.insert(key.to_string(), val);
                        }
                    } else {
                        let mut sec_table = toml::Table::new();
                        sec_table.insert(key.to_string(), val);
                        config.insert(sec.to_string(), toml::Value::Table(sec_table));
                    }
                }
                3 => {
                    let sec1 = parts[0];
                    let sec2 = parts[1];
                    let key = parts[2];
                    
                    if !config.contains_key(sec1) {
                        config.insert(sec1.to_string(), toml::Value::Table(toml::Table::new()));
                    }
                    if let Some(t1) = config.get_mut(sec1).and_then(|v| v.as_table_mut()) {
                        if !t1.contains_key(sec2) {
                            t1.insert(sec2.to_string(), toml::Value::Table(toml::Table::new()));
                        }
                        if let Some(t2) = t1.get_mut(sec2).and_then(|v| v.as_table_mut()) {
                            t2.insert(key.to_string(), val);
                        }
                    }
                }
                _ => {}
            }
        };

        let setting_item = self.noctalia_settings.iter().find(|s| s.id == id).ok_or_else(|| match self.lang {
            Language::Es => "Ajuste no reconocido".to_string(),
            Language::En => "Unrecognized setting".to_string(),
        })?;

        match setting_item.value_type {
            NoctaliaValueType::Bool => {
                let parsed: bool = value.trim().parse().map_err(|_| match self.lang {
                    Language::Es => "Valor inválido: debe ser 'true' o 'false'".to_string(),
                    Language::En => "Invalid value: must be 'true' or 'false'".to_string(),
                })?;
                update_path(id, toml::Value::Boolean(parsed));
            }
            NoctaliaValueType::Float => {
                let parsed: f64 = value.trim().parse().map_err(|_| match self.lang {
                    Language::Es => "Valor inválido: debe ser un número decimal".to_string(),
                    Language::En => "Invalid value: must be a decimal number".to_string(),
                })?;
                update_path(id, toml::Value::Float(parsed));
            }
            NoctaliaValueType::Integer => {
                let parsed: i64 = value.trim().parse().map_err(|_| match self.lang {
                    Language::Es => "Valor inválido: debe ser un número entero".to_string(),
                    Language::En => "Invalid value: must be an integer number".to_string(),
                })?;
                update_path(id, toml::Value::Integer(parsed));
            }
            NoctaliaValueType::Str => {
                let val_str = value.trim().to_string();
                if id == "dock.position" && val_str != "left" && val_str != "right" && val_str != "top" && val_str != "bottom" {
                    return Err(match self.lang {
                        Language::Es => "Valor inválido: debe ser 'left', 'right', 'top' o 'bottom'".to_string(),
                        Language::En => "Invalid value: must be 'left', 'right', 'top' or 'bottom'".to_string(),
                    });
                }
                if id == "theme.mode" && val_str != "dark" && val_str != "light" && val_str != "auto" {
                    return Err(match self.lang {
                        Language::Es => "Valor inválido: debe ser 'dark', 'light' o 'auto'".to_string(),
                        Language::En => "Invalid value: must be 'dark', 'light' or 'auto'".to_string(),
                    });
                }
                update_path(id, toml::Value::String(val_str));
            }
        }

        self.save_noctalia_config()?;
        self.reload_noctalia_settings();
        Ok(())
    }
}

/// Generates a display description of the actions inside a binds node.
pub fn get_action_desc(node: &kdl::KdlNode) -> String {
    let node_entries: Vec<String> = node.entries().iter().map(|e| e.to_string()).collect();
    let entries_suffix = if node_entries.is_empty() {
        String::new()
    } else {
        format!(" {}", node_entries.join(" "))
    };

    let main_desc = if let Some(action_doc) = node.children() {
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
        let args: Vec<String> = node.entries().iter().map(|e| e.to_string()).collect();
        args.join(" ")
    };

    if entries_suffix.is_empty() {
        main_desc
    } else {
        format!("{} {}", main_desc, entries_suffix.trim())
    }
}

/// Parses the default bundled template KDL document and returns all keybindings.
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
