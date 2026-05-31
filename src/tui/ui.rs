use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

use super::app::{ActiveOperation, ActivePanel, ActiveScreen, App};
use crate::core::i18n::Msg;
use crate::core::lua::strip_unc_prefix;

pub fn draw(f: &mut Frame, app: &App) {
    match app.active_screen {
        ActiveScreen::LanguageSelection => draw_language_selection(f, app),
        ActiveScreen::Main => draw_main(f, app),
        ActiveScreen::Settings => draw_settings(f, app),
        ActiveScreen::DirectoryInput => draw_directory_input(f, app),
    }
}

fn draw_language_selection(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 40, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Select Language / 选择语言 ")
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .padding(Padding::new(2, 2, 1, 1))
        .style(Style::default().bg(Color::DarkGray));

    let languages = vec![
        ListItem::new(Line::from(Span::raw("1. 简体中文"))),
        ListItem::new(Line::from(Span::raw("2. 繁體中文"))),
        ListItem::new(Line::from(Span::raw("3. English"))),
    ];

    let list = List::new(languages)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("● ");

    f.render_stateful_widget(list, area, &mut app.language_list_state.clone());
}

fn draw_main(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(0), // header ↔ directory
            Constraint::Length(1), // directory
            Constraint::Length(1), // directory ↔ content
            Constraint::Min(10),   // content (two bordered panels)
            Constraint::Length(1), // content ↔ command bar
            Constraint::Length(2), // command bar
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_directory_bar(f, app, chunks[2]);
    draw_content(f, app, chunks[4]);
    draw_command_bar(f, app, chunks[6]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let lang = app.language.unwrap_or(crate::core::i18n::Language::En);
    let lang_name = app.language.map(|l| l.display_name()).unwrap_or("Unknown");

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " FAOS CLI TUI ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {} ", lang.msg(Msg::CurrentLanguage)),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(lang_name, Style::default().fg(Color::White)),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Ctrl+S:{}", lang.msg(Msg::SettingsTitle)),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Q:{}", lang.msg(Msg::Quit)),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    f.render_widget(header, area);
}

fn draw_directory_bar(f: &mut Frame, app: &App, area: Rect) {
    let lang = app.language.unwrap_or(crate::core::i18n::Language::En);
    let dir_display = app
        .directory
        .as_ref()
        .map(|d| strip_unc_prefix(&d.display().to_string()).to_string())
        .unwrap_or_else(|| lang.msg(Msg::NoDirectorySelected).to_string());

    let dir_line = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {}: ", lang.msg(Msg::DirectoryLabel)),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(dir_display, Style::default().fg(Color::White)),
    ]));

    f.render_widget(dir_line, area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);
    let left_inner = left_block.inner(chunks[0]);
    f.render_widget(left_block, chunks[0]);

    let right_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);
    let right_inner = right_block.inner(chunks[1]);
    f.render_widget(right_block, chunks[1]);

    draw_file_list(f, app, left_inner);
    draw_operation_panel(f, app, right_inner);
}

fn draw_file_list(f: &mut Frame, app: &App, area: Rect) {
    let lang = app.language.unwrap_or(crate::core::i18n::Language::En);
    let files = app.get_current_files();

    let selected_count = app.selected_files.iter().filter(|s| **s).count();

    let title = match app.active_operation {
        ActiveOperation::Scan => format!(
            " {}  {}  {}/{} ",
            lang.msg(Msg::FilesLabel),
            lang.msg(Msg::ScanSettings),
            selected_count,
            files.len()
        ),
        ActiveOperation::AddAppid => format!(
            " {}  {}  {}/{} ",
            lang.msg(Msg::FilesLabel),
            lang.msg(Msg::AddAppIdSettings),
            selected_count,
            files.len()
        ),
    };

    let items: Vec<ListItem> = files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let indicator = if app.selected_files.get(i).copied().unwrap_or(false) {
                "●"
            } else {
                "○"
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", indicator),
                    if app.selected_files.get(i).copied().unwrap_or(false) {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::raw(&file.file_name),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("");

    let mut state = app.file_list_state.clone();
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_operation_panel(f: &mut Frame, app: &App, area: Rect) {
    let lang = app.language.unwrap_or(crate::core::i18n::Language::En);

    let scan_active = app.active_operation == ActiveOperation::Scan;
    let addappid_active = app.active_operation == ActiveOperation::AddAppid;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    let tab_line = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", lang.msg(Msg::F1Scan)),
            if scan_active {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {} ", lang.msg(Msg::F2AddAppId)),
            if addappid_active {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ]));

    f.render_widget(tab_line, chunks[0]);

    let separator = Paragraph::new(Line::from(Span::styled(
        "─".repeat(chunks[1].width.saturating_sub(1) as usize),
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(separator, chunks[1]);

    match app.active_operation {
        ActiveOperation::Scan => draw_scan_panel(f, app, chunks[2]),
        ActiveOperation::AddAppid => draw_addappid_panel(f, app, chunks[2]),
    }
}

fn draw_scan_panel(f: &mut Frame, app: &App, area: Rect) {
    let lang = app.language.unwrap_or(crate::core::i18n::Language::En);
    let is_active = app.active_panel == ActivePanel::OperationPanel;

    let input_value = if app.account_id.is_empty() {
        lang.msg(Msg::EnterAccountId).to_string()
    } else {
        app.account_id.clone()
    };

    let input_style = if is_active {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {}: ", lang.msg(Msg::AccountIdLabel)),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(input_value, input_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" \u{2192} ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                lang.msg(Msg::HelpExecute),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), area);

    if is_active {
        let cursor_x =
            area.x + lang.msg(Msg::AccountIdLabel).len() as u16 + 2 + app.account_id.len() as u16;
        let cursor_y = area.y;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_addappid_panel(f: &mut Frame, app: &App, area: Rect) {
    let lang = app.language.unwrap_or(crate::core::i18n::Language::En);
    let is_active = app.active_panel == ActivePanel::OperationPanel;

    let input_value = if app.app_id_input.is_empty() {
        lang.msg(Msg::EnterAppId).to_string()
    } else {
        app.app_id_input.clone()
    };

    let input_style = if is_active {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {}: ", lang.msg(Msg::AppIdLabel)),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(input_value, input_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" \u{2192} ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                lang.msg(Msg::HelpExecute),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), area);

    if is_active {
        let cursor_x =
            area.x + lang.msg(Msg::AppIdLabel).len() as u16 + 2 + app.app_id_input.len() as u16;
        let cursor_y = area.y;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_command_bar(f: &mut Frame, app: &App, area: Rect) {
    let lang = app.language.unwrap_or(crate::core::i18n::Language::En);

    let status = if app.status_message.is_empty() {
        lang.msg(Msg::Ready).to_string()
    } else {
        app.status_message.clone()
    };

    let help = match app.active_panel {
        ActivePanel::FileList => format!(
            "↑↓:{}  Space:{}  A:{}  N:{}  Tab:{}",
            lang.msg(Msg::HelpMove),
            lang.msg(Msg::HelpToggle),
            lang.msg(Msg::HelpSelectAll),
            lang.msg(Msg::HelpDeselect),
            lang.msg(Msg::HelpSwitchPanel),
        ),
        ActivePanel::OperationPanel => format!(
            "Enter:{}  Tab:{}  F5:{}",
            lang.msg(Msg::HelpExecute),
            lang.msg(Msg::HelpSwitchPanel),
            lang.msg(Msg::HelpRefresh),
        ),
    };

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(status, Style::default().fg(Color::Green)),
        Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
        Span::styled(help, Style::default().fg(Color::DarkGray)),
    ]));

    f.render_widget(bar, area);
}

fn draw_settings(f: &mut Frame, app: &App) {
    let lang = app.language.unwrap_or(crate::core::i18n::Language::En);
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let settings_text = vec![
        Line::from(Span::styled(
            format!(" {} ", lang.msg(Msg::SettingsTitle)),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "L",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("  {}", lang.msg(Msg::ChangeLanguage))),
        ]),
        Line::from(vec![
            Span::styled(
                "D",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("  {}", lang.msg(Msg::ChangeDirectory))),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            lang.msg(Msg::PressEscBack),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let settings = Paragraph::new(settings_text)
        .block(
            Block::default()
                .title(format!(" {} ", lang.msg(Msg::SettingsTitle)))
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .padding(Padding::new(2, 2, 1, 1)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(settings, area);
}

fn draw_directory_input(f: &mut Frame, app: &App) {
    let lang = app.language.unwrap_or(crate::core::i18n::Language::En);
    let area = centered_rect(70, 30, f.area());
    f.render_widget(Clear, area);

    let input_text = vec![
        Line::from(Span::styled(
            format!(" {} ", lang.msg(Msg::DirectoryLabel)),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Yellow)),
            Span::styled(&app.directory_input, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Enter: Confirm | Esc: Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let input_widget = Paragraph::new(input_text).block(
        Block::default()
            .title(format!(" {} ", lang.msg(Msg::ChangeDirectory)))
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .padding(Padding::new(2, 2, 1, 1)),
    );

    f.render_widget(input_widget, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
