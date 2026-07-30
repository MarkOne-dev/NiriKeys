use crate::app::{ActiveScreen, App, InputFocus};
use crate::translations::{Language, Translations};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, Paragraph, Wrap},
};

pub fn ui_draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Si está cargando, renderizar la pantalla de carga y retornar de inmediato
    if let ActiveScreen::Loading {
        progress,
        status_msg,
    } = &app.active_screen
    {
        frame.render_widget(Clear, size); // Limpiar pantalla completa

        let loading_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25), // Spacer superior
                Constraint::Length(5),      // ASCII Art (5 líneas)
                Constraint::Length(2),      // Espacio intermedio
                Constraint::Length(3),      // Barra de progreso y texto
                Constraint::Min(0),         // Spacer inferior
            ])
            .split(size);

        // 1. Dibujar ASCII Art centrado
        let logo = vec![
            Line::from("███    ██  ██  ██████   ██  ██   ██  ██████  ██   ██  ███████").white(),
            Line::from("████   ██  ██  ██   ██  ██  ██  ██   ██       ██ ██   ██     ").white(),
            Line::from("██ ██  ██  ██  ██████   ██  █████    █████     ███    ███████").white(),
            Line::from("██  ██ ██  ██  ██   ██  ██  ██  ██   ██         ██         ██").white(),
            Line::from("██   ████  ██  ██   ██  ██  ██   ██  ██████  ████     ███████").white(),
        ];
        let logo_widget = Paragraph::new(logo).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(logo_widget, loading_layout[1]);

        // 2. Dibujar Barra de Progreso y Mensaje
        let filled_width = ((*progress as usize) * 30) / 100;
        let empty_width = 30 - filled_width;
        let bar = format!(
            " [{}{}] {}%",
            "█".repeat(filled_width),
            "░".repeat(empty_width),
            progress
        );

        let progress_lines = vec![
            Line::from(status_msg.clone()).dim().white(),
            Line::from(""),
            Line::from(bar).cyan().bold(),
        ];
        let progress_widget =
            Paragraph::new(progress_lines).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(progress_widget, loading_layout[3]);
        return;
    }

    // 1. Layout Principal (Header, Dashboard, Footer)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Dashboard
            Constraint::Length(1), // Footer / Leyenda
        ])
        .split(size);

    // Renderizar Header
    let header_widget = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            Translations::get(&app.lang).title,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            Translations::get(&app.lang).subtitle,
            Style::default().fg(Color::Cyan),
        ),
        if app.dry_run {
            Span::styled(
                Translations::get(&app.lang).dry_run,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
    ])])
    .block(Block::bordered().border_style(Style::default().fg(Color::Magenta)));
    frame.render_widget(header_widget, chunks[0]);

    // Renderizar Footer (Guía rápida de teclas)
    let help_text = match app.active_screen {
        ActiveScreen::Dashboard => {
            if app.active_tab == 0 {
                Translations::get(&app.lang).help_dashboard
            } else {
                Translations::get(&app.lang).help_appearance
            }
        }
        ActiveScreen::CreateConfigPrompt => Translations::get(&app.lang).help_create_config,
        ActiveScreen::AddPopup => Translations::get(&app.lang).help_add,
        ActiveScreen::ConfirmOverwrite { .. } => Translations::get(&app.lang).help_confirm,
        ActiveScreen::ErrorPopup(_) | ActiveScreen::InfoPopup(_) => {
            Translations::get(&app.lang).help_any_key
        }
        ActiveScreen::InstallPrompt { .. } => Translations::get(&app.lang).help_install,
        ActiveScreen::MergePopup { .. } => Translations::get(&app.lang).help_merge,
        ActiveScreen::EditAppearancePopup { .. } => {
            Translations::get(&app.lang).modal_appearance_guide
        }
        ActiveScreen::Loading { .. } => "",
    };
    let footer_widget = Paragraph::new(help_text).style(
        Style::default()
            .bg(Color::Magenta)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(footer_widget, chunks[2]);

    // 1.5. Dividir Dashboard en Tabs y Contenido
    let dashboard_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(2),    // Dashboard Columns
        ])
        .split(chunks[1]);

    // Renderizar Tab bar
    let tab_titles = vec![
        Translations::get(&app.lang).tab_shortcuts,
        Translations::get(&app.lang).tab_appearance,
    ];
    let tab_spans = tab_titles
        .iter()
        .enumerate()
        .map(|(idx, title)| {
            if idx == app.active_tab {
                Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!(" {} ", title), Style::default().fg(Color::Gray))
            }
        })
        .collect::<Vec<_>>();

    let mut tab_line = Vec::new();
    for (i, span) in tab_spans.into_iter().enumerate() {
        if i > 0 {
            tab_line.push(Span::raw("   "));
        }
        tab_line.push(span);
    }

    let tabs_widget = Paragraph::new(Line::from(tab_line)).block(
        Block::bordered()
            .title(" Menú ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(tabs_widget, dashboard_chunks[0]);

    // 2. Renderizar Dashboard Principal (Dos Columnas)
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Columna Izquierda: Metadatos y ayuda
            Constraint::Percentage(65), // Columna Derecha: Lista o Apariencia
        ])
        .split(dashboard_chunks[1]);

    // Columna Izquierda (Metadatos & Info de Niri)
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // Metadatos Card
            Constraint::Min(4),    // Panel Informativo
        ])
        .split(main_layout[0]);

    // Renderizar Metadatos Card
    let status_style = if app.file_is_valid {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };

    let status_text = if app.file_is_valid {
        Translations::get(&app.lang).valid
    } else {
        Translations::get(&app.lang).invalid
    };

    let metadata_text = vec![
        Line::from(vec![
            Span::styled(
                Translations::get(&app.lang).path,
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                app.config_path.to_string_lossy(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                Translations::get(&app.lang).size,
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{:.2} KB", app.file_size_kb),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                Translations::get(&app.lang).modif,
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(&app.file_mod_time, Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled(
                Translations::get(&app.lang).syntax,
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(status_text, status_style),
        ]),
    ];

    let metadata_card = Paragraph::new(metadata_text).block(
        Block::bordered()
            .title(Translations::get(&app.lang).entorno_title)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(metadata_card, left_chunks[0]);

    // Renderizar Panel Informativo Inferior
    let info_text = vec![
        Line::from(Translations::get(&app.lang).info_line1),
        Line::from(Translations::get(&app.lang).info_line2),
        Line::from(Translations::get(&app.lang).info_line3),
        Line::from(Translations::get(&app.lang).info_line4),
    ];
    let info_card = Paragraph::new(info_text).block(
        Block::bordered()
            .title(Translations::get(&app.lang).info_title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(info_card, left_chunks[1]);

    // Columna Derecha: Renderizar Lista de Atajos o Editor de Apariencia
    if app.active_tab == 0 {
        let list_items: Vec<ListItem> = app
            .keybindings
            .iter()
            .map(|(key, action)| {
                ListItem::new(vec![Line::from(vec![
                    Span::styled(
                        format!("  {:width$} ", key, width = 20),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(action.clone(), Style::default().fg(Color::White)),
                ])])
            })
            .collect();

        let list_widget = List::new(list_items)
            .block(
                Block::bordered()
                    .title(format!(
                        " {} ({}) ",
                        Translations::get(&app.lang).list_title,
                        app.keybindings.len()
                    ))
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(40, 44, 52))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("  ");

        frame.render_stateful_widget(list_widget, main_layout[1], &mut app.list_state);
    } else {
        let settings = app.get_appearance_settings();
        let list_items: Vec<ListItem> = settings
            .iter()
            .map(|setting| {
                ListItem::new(vec![Line::from(vec![
                    Span::styled(
                        format!("  {:width$} ", setting.name, width = 35),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(setting.value.clone(), Style::default().fg(Color::White)),
                ])])
            })
            .collect();

        let list_widget = List::new(list_items)
            .block(
                Block::bordered()
                    .title(" Configuración Estética ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(40, 44, 52))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("  ");

        frame.render_stateful_widget(list_widget, main_layout[1], &mut app.appearance_state);
    }

    // 3. Renderizar Popups Modales (si están activos)
    match &app.active_screen {
        ActiveScreen::InstallPrompt { pm_name, cmd } => {
            let popup_area = get_centered_rect(55, 30, size);
            frame.render_widget(Clear, popup_area);

            let prompt_text = vec![
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_install_msg1),
                Line::from(""),
                Line::from(
                    Translations::get(&app.lang)
                        .modal_install_msg2
                        .replace("{}", pm_name),
                ),
                Line::from(""),
                Line::from(
                    Translations::get(&app.lang)
                        .modal_install_msg3
                        .replace("{}", cmd),
                ),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "   [i] ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        match app.lang {
                            Language::Es => "Instalar   ",
                            Language::En => "Install   ",
                        },
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(
                        "   [Esc/q] ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        match app.lang {
                            Language::Es => "Salir",
                            Language::En => "Exit",
                        },
                        Style::default().fg(Color::Red),
                    ),
                ]),
            ];

            let popup_block = Block::bordered()
                .title(Translations::get(&app.lang).modal_install_title)
                .border_style(Style::default().fg(Color::Yellow));

            let prompt_paragraph = Paragraph::new(prompt_text)
                .block(popup_block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(prompt_paragraph, popup_area);
        }
        ActiveScreen::CreateConfigPrompt => {
            let popup_area = get_centered_rect(50, 25, size);
            frame.render_widget(Clear, popup_area);

            let prompt_text = vec![
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_create_msg1),
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_create_msg2),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        Translations::get(&app.lang).modal_create_yes,
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        Translations::get(&app.lang).modal_create_no,
                        Style::default().fg(Color::Red),
                    ),
                ]),
            ];

            let popup_block = Block::bordered()
                .title(Translations::get(&app.lang).modal_create_title)
                .border_style(Style::default().fg(Color::Yellow));
            let prompt_paragraph = Paragraph::new(prompt_text)
                .block(popup_block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(prompt_paragraph, popup_area);
        }
        ActiveScreen::AddPopup => {
            let popup_area = get_centered_rect(60, 35, size);
            frame.render_widget(Clear, popup_area);

            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Input Key
                    Constraint::Length(3), // Input Action
                    Constraint::Min(1),    // Guía del modal
                ])
                .split(popup_area);

            // Caja para Teclas
            let key_style = if app.input_focus == InputFocus::Key {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let key_block = Block::bordered()
                .title(Translations::get(&app.lang).modal_add_key_title)
                .border_style(key_style);

            // Cursor simulado
            let key_cursor = if app.input_focus == InputFocus::Key {
                "_"
            } else {
                ""
            };
            let key_text = format!("{}{}", app.input_key, key_cursor);
            let key_widget = Paragraph::new(key_text).block(key_block);
            frame.render_widget(key_widget, popup_layout[0]);

            // Caja para Acción
            let action_style = if app.input_focus == InputFocus::Action {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let action_block = Block::bordered()
                .title(Translations::get(&app.lang).modal_add_action_title)
                .border_style(action_style);

            let action_cursor = if app.input_focus == InputFocus::Action {
                "_"
            } else {
                ""
            };
            let action_text = format!("{}{}", app.input_action, action_cursor);
            let action_widget = Paragraph::new(action_text).block(action_block);
            frame.render_widget(action_widget, popup_layout[1]);

            // Guía del modal
            let modal_guide = Paragraph::new(Translations::get(&app.lang).modal_add_guide)
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(modal_guide, popup_layout[2]);

            // Dibujar el borde del popup general alrededor de los inputs
            let outer_block = Block::bordered()
                .title(Translations::get(&app.lang).modal_add_outer_title)
                .border_style(Style::default().fg(Color::Magenta));
            frame.render_widget(outer_block, popup_area);
        }
        ActiveScreen::ConfirmOverwrite { key: k, .. } => {
            let popup_area = get_centered_rect(50, 25, size);
            frame.render_widget(Clear, popup_area);

            let prompt_text = vec![
                Line::from(""),
                Line::from(
                    Translations::get(&app.lang)
                        .modal_confirm_msg1
                        .replace("{}", &k),
                ),
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_confirm_msg2),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        Translations::get(&app.lang).modal_confirm_yes,
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        Translations::get(&app.lang).modal_confirm_no,
                        Style::default().fg(Color::Red),
                    ),
                ]),
            ];

            let popup_block = Block::bordered()
                .title(Translations::get(&app.lang).modal_confirm_title)
                .border_style(Style::default().fg(Color::Yellow));
            let prompt_paragraph = Paragraph::new(prompt_text)
                .block(popup_block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(prompt_paragraph, popup_area);
        }
        ActiveScreen::ErrorPopup(err_text) => {
            let popup_area = get_centered_rect(70, 50, size);
            frame.render_widget(Clear, popup_area);

            let error_block = Block::bordered()
                .title(Translations::get(&app.lang).modal_error_title)
                .border_style(Style::default().fg(Color::Red));

            let wrapped_text = vec![
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_error_msg.red()),
                Line::from(""),
            ];

            let mut final_text = wrapped_text;
            for line in err_text.lines() {
                final_text.push(Line::from(line.yellow().to_string()));
            }
            final_text.push(Line::from(""));
            final_text.push(Line::from(
                Translations::get(&app.lang)
                    .modal_error_close
                    .dim()
                    .white()
                    .to_string(),
            ));

            let error_paragraph = Paragraph::new(final_text)
                .block(error_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(error_paragraph, popup_area);
        }
        ActiveScreen::InfoPopup(info_text) => {
            let popup_area = get_centered_rect(50, 25, size);
            frame.render_widget(Clear, popup_area);

            let popup_block = Block::bordered()
                .title(Translations::get(&app.lang).modal_info_title)
                .border_style(Style::default().fg(Color::Green));

            let mut lines = vec![Line::from("")];
            for line in info_text.lines() {
                lines.push(Line::from(line.white().to_string()));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(
                Translations::get(&app.lang)
                    .modal_info_close
                    .dim()
                    .white()
                    .to_string(),
            ));

            let info_paragraph = Paragraph::new(lines)
                .block(popup_block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(info_paragraph, popup_area);
        }
        ActiveScreen::MergePopup {
            missing,
            selected_idx,
        } => {
            let popup_area = get_centered_rect(70, 60, size);
            frame.render_widget(Clear, popup_area);

            let popup_block = Block::bordered()
                .title(match app.lang {
                    Language::Es => " 󰅩 Importar Atajos Recomendados ",
                    Language::En => " 󰅩 Import Recommended Shortcuts ",
                })
                .border_style(Style::default().fg(Color::Yellow));

            let mut lines = vec![
                Line::from(""),
                Line::from(match app.lang {
                    Language::Es => "Se encontraron los siguientes atajos en la plantilla oficial que no tienes configurados.",
                    Language::En => "The following shortcuts from the official template are not configured in your file.",
                }).dim(),
                Line::from(""),
            ];

            // Renderizar la lista de atajos con scroll/highlight
            for (idx, (key, action)) in missing.iter().enumerate() {
                let is_selected = idx == *selected_idx;
                let prefix = if is_selected { "  " } else { "   " };

                let key_span = Span::styled(
                    format!("{:<20}", key),
                    if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    },
                );
                let action_span = Span::styled(
                    action,
                    if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                );

                let style = if is_selected {
                    Style::default().bg(Color::Rgb(40, 44, 52))
                } else {
                    Style::default()
                };

                lines.push(
                    Line::from(vec![
                        Span::styled(
                            prefix,
                            if is_selected {
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
                            },
                        ),
                        key_span,
                        Span::raw("   "),
                        action_span,
                    ])
                    .style(style),
                );
            }

            lines.push(Line::from(""));
            lines.push(Line::from(match app.lang {
                Language::Es => " [Enter/i] Importar seleccionado   [y] Importar TODOS de un porrazo   [Esc] Cancelar ",
                Language::En => " [Enter/i] Import selected   [y] Import ALL at once   [Esc] Cancel ",
            }).bold().cyan());

            let paragraph = Paragraph::new(lines)
                .block(popup_block)
                .wrap(Wrap { trim: false });
            frame.render_widget(paragraph, popup_area);
        }
        ActiveScreen::EditAppearancePopup {
            setting_id: _,
            setting_name,
            input_value,
        } => {
            let popup_area = get_centered_rect(60, 25, size);
            frame.render_widget(Clear, popup_area);

            let popup_block = Block::bordered()
                .title(Translations::get(&app.lang).modal_appearance_title)
                .border_style(Style::default().fg(Color::Yellow));

            let prompt_text = vec![
                Line::from(""),
                Line::from(
                    Translations::get(&app.lang)
                        .modal_appearance_msg
                        .replace("{}", setting_name),
                ),
                Line::from(""),
                Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(
                        format!("{}_", input_value),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_appearance_guide).dim(),
            ];

            let prompt_paragraph = Paragraph::new(prompt_text)
                .block(popup_block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(prompt_paragraph, popup_area);
        }
        ActiveScreen::Dashboard | ActiveScreen::Loading { .. } => {}
    }
}

fn get_centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
