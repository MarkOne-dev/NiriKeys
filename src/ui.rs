use crate::app::{ActiveScreen, App, InputFocus};
use crate::translations::{Language, Translations};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, Paragraph, Wrap},
};

/// Draws the entire TUI application interface based on the active screen state.
pub fn ui_draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Color Palette Constants
    let bg_main = Color::Rgb(15, 14, 29);         // #0f0e1d
    let bg_sidebar = Color::Rgb(21, 20, 43);      // #15142b
    let accent_yellow = Color::Rgb(245, 224, 107); // #f5e06b
    let color_border = Color::Rgb(42, 39, 74);    // #2a274a
    let color_text_main = Color::White;           // #ffffff
    let color_text_sec = Color::Rgb(176, 174, 196); // #b0aec4

    // Render overall main background
    frame.render_widget(Block::new().bg(bg_main), size);
    
    let get_display_path = |path: &std::path::Path| -> String {
        if let Some(home) = dirs::home_dir() {
            if let Ok(stripped) = path.strip_prefix(&home) {
                return format!("~/{}", stripped.display());
            }
        }
        path.to_string_lossy().into_owned()
    };

    if let ActiveScreen::Loading {
        progress,
        status_msg,
    } = &app.active_screen
    {
        frame.render_widget(Clear, size);

        let loading_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Length(5),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(size);

        let logo = vec![
            Line::from("███    ██  ██  ██████   ██  ██   ██  ██████  ██   ██  ███████").white(),
            Line::from("████   ██  ██  ██   ██  ██  ██  ██   ██       ██ ██   ██     ").white(),
            Line::from("██ ██  ██  ██  ██████   ██  █████    █████     ███    ███████").white(),
            Line::from("██  ██ ██  ██  ██   ██  ██  ██  ██   ██         ██         ██").white(),
            Line::from("██   ████  ██  ██   ██  ██  ██   ██  ██████  ████     ███████").white(),
        ];
        let logo_widget = Paragraph::new(logo).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(logo_widget, loading_layout[1]);

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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(size);

    let header_widget = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            Translations::get(&app.lang).title,
            Style::default()
                .fg(accent_yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            Translations::get(&app.lang).subtitle,
            Style::default().fg(color_text_sec),
        ),
        if app.dry_run {
            Span::styled(
                Translations::get(&app.lang).dry_run,
                Style::default()
                    .fg(accent_yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
    ])])
    .block(
        Block::bordered()
            .border_style(Style::default().fg(color_border))
            .bg(bg_sidebar),
    );
    frame.render_widget(header_widget, chunks[0]);

    let help_text = match app.active_screen {
        ActiveScreen::Dashboard => {
            if app.active_tab == 0 {
                Translations::get(&app.lang).help_dashboard
            } else if app.active_tab == 1 {
                Translations::get(&app.lang).help_appearance
            } else {
                Translations::get(&app.lang).help_noctalia
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
        ActiveScreen::EditNoctaliaPopup { .. } => {
            Translations::get(&app.lang).modal_appearance_guide
        }
        ActiveScreen::Loading { .. } => "",
    };
    let footer_widget = Paragraph::new(help_text).style(
        Style::default()
            .bg(bg_sidebar)
            .fg(color_text_sec)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(footer_widget, chunks[2]);

    let dashboard_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(2)])
        .split(chunks[1]);

    let tab_titles = vec![
        Translations::get(&app.lang).tab_shortcuts,
        Translations::get(&app.lang).tab_appearance,
        Translations::get(&app.lang).tab_noctalia,
    ];
    let tab_spans = tab_titles
        .iter()
        .enumerate()
        .map(|(idx, title)| {
            if idx == app.active_tab {
                Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .fg(bg_main)
                        .bg(accent_yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!(" {} ", title), Style::default().fg(color_text_sec))
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
            .title(Span::styled(" Menú ", Style::default().fg(color_text_main)))
            .border_style(Style::default().fg(color_border))
            .bg(bg_sidebar),
    );
    frame.render_widget(tabs_widget, dashboard_chunks[0]);

    let main_vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(dashboard_chunks[1]);

    let upper_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main_vertical_chunks[0]);

    let left_upper_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(3)])
        .split(upper_layout[0]);

    let (metadata_text, metadata_title, metadata_border_color) = if app.active_tab == 2 {
        let status_style = if app.noctalia_config.is_some() && app.noctalia_is_valid {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        };

        let status_text = if !app.noctalia_path.exists() {
            Translations::get(&app.lang).noctalia_not_found
        } else if app.noctalia_is_valid {
            Translations::get(&app.lang).valid
        } else {
            Translations::get(&app.lang).noctalia_invalid
        };

        (
            vec![
                Line::from(vec![
                    Span::styled(
                        Translations::get(&app.lang).path,
                        Style::default().fg(color_text_sec),
                    ),
                    Span::styled(
                        get_display_path(&app.noctalia_path),
                        Style::default().fg(color_text_main),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        Translations::get(&app.lang).syntax,
                        Style::default().fg(color_text_sec),
                    ),
                    Span::styled(status_text, status_style),
                ]),
            ],
            Translations::get(&app.lang).noctalia_entorno_title,
            color_border,
        )
    } else {
        let status_style = if app.file_is_valid {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        };

        let status_text = if app.file_is_valid {
            Translations::get(&app.lang).valid
        } else {
            Translations::get(&app.lang).invalid
        };

        (
            vec![
                Line::from(vec![
                    Span::styled(
                        Translations::get(&app.lang).path,
                        Style::default().fg(color_text_sec),
                    ),
                    Span::styled(
                        get_display_path(&app.config_path),
                        Style::default().fg(color_text_main),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        Translations::get(&app.lang).size,
                        Style::default().fg(color_text_sec),
                    ),
                    Span::styled(
                        format!("{:.2} KB", app.file_size_kb),
                        Style::default().fg(accent_yellow),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        Translations::get(&app.lang).modif,
                        Style::default().fg(color_text_sec),
                    ),
                    Span::styled(&app.file_mod_time, Style::default().fg(color_text_main)),
                ]),
                Line::from(vec![
                    Span::styled(
                        Translations::get(&app.lang).syntax,
                        Style::default().fg(color_text_sec),
                    ),
                    Span::styled(status_text, status_style),
                ]),
            ],
            Translations::get(&app.lang).entorno_title,
            color_border,
        )
    };

    let metadata_card = Paragraph::new(metadata_text).block(
        Block::bordered()
            .title(Span::styled(metadata_title, Style::default().fg(color_text_main)))
            .border_style(Style::default().fg(metadata_border_color))
            .bg(bg_sidebar),
    );
    frame.render_widget(metadata_card, left_upper_chunks[0]);

    // --- LIVE FILE PREVIEW ---
    let preview_raw = app.get_live_file_preview();
    let preview_lines: Vec<Line> = preview_raw
        .lines()
        .enumerate()
        .map(|(i, line)| {
            Line::from(vec![
                Span::styled(format!("{:3} │ ", i + 1), Style::default().fg(color_text_sec)),
                Span::styled(line, Style::default().fg(color_text_main)),
            ])
        })
        .collect();

    let preview_height = main_vertical_chunks[1].height as usize;
    let max_scroll = if preview_lines.len() > preview_height.saturating_sub(2) {
        (preview_lines.len() - (preview_height.saturating_sub(2))) as u16
    } else {
        0
    };
    if app.preview_scroll > max_scroll {
        app.preview_scroll = max_scroll;
    }

    let preview_card = Paragraph::new(preview_lines)
        .scroll((app.preview_scroll, 0))
        .block(
            Block::bordered()
                .title(Span::styled(
                    match app.lang {
                        Language::Es => " Previsualización en Vivo [ [ / ] ] ",
                        Language::En => " Live Preview [ [ / ] ] ",
                    },
                    Style::default().fg(color_text_main),
                ))
                .border_style(Style::default().fg(color_border))
                .bg(bg_main),
        );
    frame.render_widget(preview_card, main_vertical_chunks[1]);

    // --- AGENT ACTIVITY LOGS ---
    let log_lines: Vec<Line> = app
        .agent_logs
        .iter()
        .map(|log| {
            let parts: Vec<&str> = log.splitn(2, ']').collect();
            if parts.len() == 2 {
                Line::from(vec![
                    Span::styled(format!("{}]", parts[0]), Style::default().fg(color_text_sec)),
                    Span::styled(parts[1].to_string(), Style::default().fg(accent_yellow)),
                ])
            } else {
                Line::from(vec![Span::raw(log)])
            }
        })
        .collect();

    let logs_height = left_upper_chunks[1].height as usize;
    let log_scroll = if log_lines.len() > logs_height.saturating_sub(2) {
        (log_lines.len() - (logs_height.saturating_sub(2))) as u16
    } else {
        0
    };

    let logs_card = Paragraph::new(log_lines)
        .scroll((log_scroll, 0))
        .block(
            Block::bordered()
                .title(Span::styled(
                    match app.lang {
                        Language::Es => " Bitácora del Agente ",
                        Language::En => " Agent Activity Log ",
                    },
                    Style::default().fg(color_text_main),
                ))
                .border_style(Style::default().fg(color_border))
                .bg(bg_sidebar),
        );
    frame.render_widget(logs_card, left_upper_chunks[1]);

    if app.active_tab == 0 {
        let list_items: Vec<ListItem> = app
            .keybindings
            .iter()
            .map(|(key, action)| {
                ListItem::new(vec![Line::from(vec![
                    Span::styled(
                        format!("  {:width$} ", key, width = 20),
                        Style::default()
                            .fg(accent_yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("   ", Style::default().fg(color_text_sec)),
                    Span::styled(action.clone(), Style::default().fg(color_text_main)),
                ])])
            })
            .collect();

        let list_widget = List::new(list_items)
            .block(
                Block::bordered()
                    .title(Span::styled(
                        format!(
                            " {} ({}) ",
                            Translations::get(&app.lang).list_title,
                            app.keybindings.len()
                        ),
                        Style::default().fg(color_text_main),
                    ))
                    .border_style(Style::default().fg(color_border))
                    .bg(bg_main),
            )
            .highlight_style(
                Style::default()
                    .bg(bg_sidebar)
                    .fg(accent_yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("  ");

        frame.render_stateful_widget(list_widget, upper_layout[1], &mut app.list_state);
    } else if app.active_tab == 1 {
        let settings = app.get_appearance_settings();
        let list_items: Vec<ListItem> = settings
            .iter()
            .map(|setting| {
                ListItem::new(vec![Line::from(vec![
                    Span::styled(
                        format!("  {:width$} ", setting.name, width = 35),
                        Style::default()
                            .fg(color_text_sec)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("   ", Style::default().fg(color_text_sec)),
                    Span::styled(setting.value.clone(), Style::default().fg(accent_yellow)),
                ])])
            })
            .collect();

        let list_widget = List::new(list_items)
            .block(
                Block::bordered()
                    .title(Span::styled(" Configuración Estética ", Style::default().fg(color_text_main)))
                    .border_style(Style::default().fg(color_border))
                    .bg(bg_main),
            )
            .highlight_style(
                Style::default()
                    .bg(bg_sidebar)
                    .fg(accent_yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("  ");

        frame.render_stateful_widget(list_widget, upper_layout[1], &mut app.appearance_state);
    } else {
        let list_items: Vec<ListItem> = app
            .noctalia_settings
            .iter()
            .map(|setting| {
                ListItem::new(vec![Line::from(vec![
                    Span::styled(
                        format!("  {:width$} ", setting.name, width = 45),
                        Style::default()
                            .fg(color_text_sec)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("   ", Style::default().fg(color_text_sec)),
                    Span::styled(setting.value.clone(), Style::default().fg(accent_yellow)),
                ])])
            })
            .collect();

        let list_widget = List::new(list_items)
            .block(
                Block::bordered()
                    .title(Span::styled(" Ajustes de Noctalia UI ", Style::default().fg(color_text_main)))
                    .border_style(Style::default().fg(color_border))
                    .bg(bg_main),
            )
            .highlight_style(
                Style::default()
                    .bg(bg_sidebar)
                    .fg(accent_yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("  ");

        frame.render_stateful_widget(list_widget, upper_layout[1], &mut app.noctalia_state);
    }

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
                .title(Span::styled(Translations::get(&app.lang).modal_install_title, Style::default().fg(color_text_main)))
                .border_style(Style::default().fg(color_border))
                .bg(bg_sidebar);

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
                Line::from(Translations::get(&app.lang).modal_create_msg1).style(Style::default().fg(color_text_main)),
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_create_msg2).style(Style::default().fg(color_text_sec)),
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
                .title(Span::styled(Translations::get(&app.lang).modal_create_title, Style::default().fg(color_text_main)))
                .border_style(Style::default().fg(color_border))
                .bg(bg_sidebar);
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
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(popup_area);

            let key_style = if app.input_focus == InputFocus::Key {
                Style::default().fg(accent_yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color_border)
            };
            let key_block = Block::bordered()
                .title(Span::styled(Translations::get(&app.lang).modal_add_key_title, Style::default().fg(color_text_main)))
                .border_style(key_style);

            let key_cursor = if app.input_focus == InputFocus::Key {
                "_"
            } else {
                ""
            };
            let key_text = format!("{}{}", app.input_key, key_cursor);
            let key_widget = Paragraph::new(key_text).block(key_block);
            frame.render_widget(key_widget, popup_layout[0]);

            let action_style = if app.input_focus == InputFocus::Action {
                Style::default().fg(accent_yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color_border)
            };
            let action_block = Block::bordered()
                .title(Span::styled(Translations::get(&app.lang).modal_add_action_title, Style::default().fg(color_text_main)))
                .border_style(action_style);

            let action_cursor = if app.input_focus == InputFocus::Action {
                "_"
            } else {
                ""
            };
            let action_text = format!("{}{}", app.input_action, action_cursor);
            let action_widget = Paragraph::new(action_text).block(action_block);
            frame.render_widget(action_widget, popup_layout[1]);

            let modal_guide = Paragraph::new(Translations::get(&app.lang).modal_add_guide)
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(color_text_sec));
            frame.render_widget(modal_guide, popup_layout[2]);

            let outer_block = Block::bordered()
                .title(Span::styled(Translations::get(&app.lang).modal_add_outer_title, Style::default().fg(color_text_main)))
                .border_style(Style::default().fg(color_border))
                .bg(bg_sidebar);
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
                ).style(Style::default().fg(color_text_main)),
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_confirm_msg2).style(Style::default().fg(color_text_sec)),
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
                .title(Span::styled(Translations::get(&app.lang).modal_confirm_title, Style::default().fg(color_text_main)))
                .border_style(Style::default().fg(color_border))
                .bg(bg_sidebar);
            let prompt_paragraph = Paragraph::new(prompt_text)
                .block(popup_block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(prompt_paragraph, popup_area);
        }
        ActiveScreen::ErrorPopup(err_text) => {
            let popup_area = get_centered_rect(70, 50, size);
            frame.render_widget(Clear, popup_area);

            let error_block = Block::bordered()
                .title(Span::styled(Translations::get(&app.lang).modal_error_title, Style::default().fg(Color::Red)))
                .border_style(Style::default().fg(Color::Red))
                .bg(bg_sidebar);

            let wrapped_text = vec![
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_error_msg.red().bold()),
                Line::from(""),
            ];

            let mut final_text = wrapped_text;
            for line in err_text.lines() {
                final_text.push(Line::from(line.to_string()).style(Style::default().fg(accent_yellow)));
            }
            final_text.push(Line::from(""));
            final_text.push(Line::from(
                Translations::get(&app.lang)
                    .modal_error_close
                    .dim()
                    .white()
                    .to_string(),
            ).style(Style::default().fg(color_text_sec)));

            let error_paragraph = Paragraph::new(final_text)
                .block(error_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(error_paragraph, popup_area);
        }
        ActiveScreen::InfoPopup(info_text) => {
            let popup_area = get_centered_rect(50, 25, size);
            frame.render_widget(Clear, popup_area);

            let popup_block = Block::bordered()
                .title(Span::styled(Translations::get(&app.lang).modal_info_title, Style::default().fg(color_text_main)))
                .border_style(Style::default().fg(color_border))
                .bg(bg_sidebar);

            let mut lines = vec![Line::from("")];
            for line in info_text.lines() {
                lines.push(Line::from(line.to_string()).style(Style::default().fg(color_text_main)));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(
                Translations::get(&app.lang)
                    .modal_info_close
                    .dim()
                    .white()
                    .to_string(),
            ).style(Style::default().fg(color_text_sec)));

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
                .title(Span::styled(
                    match app.lang {
                        Language::Es => " 󰅩 Importar Atajos Recomendados ",
                        Language::En => " 󰅩 Import Recommended Shortcuts ",
                    },
                    Style::default().fg(color_text_main),
                ))
                .border_style(Style::default().fg(color_border))
                .bg(bg_sidebar);

            let mut lines = vec![
                Line::from(""),
                Line::from(match app.lang {
                    Language::Es => "Se encontraron los siguientes atajos en la plantilla oficial que no tienes configurados.",
                    Language::En => "The following shortcuts from the official template are not configured in your file.",
                }).style(Style::default().fg(color_text_sec)),
                Line::from(""),
            ];

            for (idx, (key, action)) in missing.iter().enumerate() {
                let is_selected = idx == *selected_idx;
                let prefix = if is_selected { "  " } else { "   " };

                let key_span = Span::styled(
                    format!("{:<20}", key),
                    if is_selected {
                        Style::default()
                            .fg(accent_yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color_text_main)
                    },
                );
                let action_span = Span::styled(
                    action,
                    if is_selected {
                        Style::default()
                            .fg(accent_yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color_text_sec)
                    },
                );

                let style = if is_selected {
                    Style::default().bg(bg_main)
                } else {
                    Style::default()
                };

                lines.push(
                    Line::from(vec![
                        Span::styled(
                            prefix,
                            if is_selected {
                                Style::default()
                                    .fg(accent_yellow)
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
            }).bold().fg(accent_yellow));

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
                .title(Span::styled(Translations::get(&app.lang).modal_appearance_title, Style::default().fg(color_text_main)))
                .border_style(Style::default().fg(color_border))
                .bg(bg_sidebar);

            let prompt_text = vec![
                Line::from(""),
                Line::from(
                    Translations::get(&app.lang)
                        .modal_appearance_msg
                        .replace("{}", setting_name),
                ).style(Style::default().fg(color_text_main)),
                Line::from(""),
                Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(
                        format!("{}_", input_value),
                        Style::default()
                            .fg(accent_yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_appearance_guide).style(Style::default().fg(color_text_sec)),
            ];

            let prompt_paragraph = Paragraph::new(prompt_text)
                .block(popup_block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(prompt_paragraph, popup_area);
        }
        ActiveScreen::EditNoctaliaPopup {
            setting_id: _,
            setting_name,
            input_value,
            value_type: _,
        } => {
            let popup_area = get_centered_rect(60, 25, size);
            frame.render_widget(Clear, popup_area);

            let popup_block = Block::bordered()
                .title(Span::styled(Translations::get(&app.lang).modal_noctalia_title, Style::default().fg(color_text_main)))
                .border_style(Style::default().fg(color_border))
                .bg(bg_sidebar);

            let prompt_text = vec![
                Line::from(""),
                Line::from(
                    Translations::get(&app.lang)
                        .modal_noctalia_msg
                        .replace("{}", setting_name),
                ).style(Style::default().fg(color_text_main)),
                Line::from(""),
                Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(
                        format!("{}_", input_value),
                        Style::default()
                            .fg(accent_yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(Translations::get(&app.lang).modal_appearance_guide).style(Style::default().fg(color_text_sec)),
            ];

            let prompt_paragraph = Paragraph::new(prompt_text)
                .block(popup_block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(prompt_paragraph, popup_area);
        }
        ActiveScreen::Dashboard | ActiveScreen::Loading { .. } => {}
    }
}

/// Helper function to calculate a centered rectangle layout.
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
