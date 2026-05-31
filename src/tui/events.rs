use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::Duration;

use super::app::{ActiveOperation, ActivePanel, ActiveScreen, App};
use crate::core::i18n::Language;

pub fn handle_events(app: &mut App) -> std::io::Result<()> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key_event) => {
                // 只处理按键按下事件，忽略释放事件
                if key_event.kind == KeyEventKind::Press {
                    handle_key_event(app, key_event);
                }
            }
            Event::Mouse(mouse_event) => handle_mouse_event(app, mouse_event),
            _ => {}
        }
    }
    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) {
    match app.active_screen {
        ActiveScreen::LanguageSelection => handle_language_selection_keys(app, key),
        ActiveScreen::Main => handle_main_keys(app, key),
        ActiveScreen::Settings => handle_settings_keys(app, key),
        ActiveScreen::DirectoryInput => handle_directory_input_keys(app, key),
    }
}

fn handle_language_selection_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(selected) = app.language_list_state.selected() {
                if selected > 0 {
                    app.language_list_state.select(Some(selected - 1));
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(selected) = app.language_list_state.selected() {
                if selected < 2 {
                    app.language_list_state.select(Some(selected + 1));
                }
            }
        }
        KeyCode::Enter => {
            if let Some(selected) = app.language_list_state.selected() {
                let language = match selected {
                    0 => Language::ZhCn,
                    1 => Language::ZhTw,
                    2 => Language::En,
                    _ => return,
                };
                app.select_language(language);
            }
        }
        KeyCode::Char('1') => app.select_language(Language::ZhCn),
        KeyCode::Char('2') => app.select_language(Language::ZhTw),
        KeyCode::Char('3') => app.select_language(Language::En),
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        _ => {}
    }
}

fn handle_main_keys(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('q') => app.quit(),
            KeyCode::Char('s') => {
                app.active_screen = ActiveScreen::Settings;
            }
            _ => {}
        }
        return;
    }

    // F1/F2/F5 在任何面板都生效
    match key.code {
        KeyCode::F(1) => {
            app.switch_operation(ActiveOperation::Scan);
            return;
        }
        KeyCode::F(2) => {
            app.switch_operation(ActiveOperation::AddAppid);
            return;
        }
        KeyCode::F(5) => {
            app.load_files();
            return;
        }
        _ => {}
    }

    match key.code {
        KeyCode::Tab => {
            app.active_panel = match app.active_panel {
                ActivePanel::FileList => ActivePanel::OperationPanel,
                ActivePanel::OperationPanel => ActivePanel::FileList,
            };
        }
        KeyCode::Esc => {
            if app.active_panel == ActivePanel::OperationPanel {
                app.active_panel = ActivePanel::FileList;
            } else {
                app.quit();
            }
        }
        KeyCode::Char('q') => app.quit(),
        _ => {}
    }

    match app.active_panel {
        ActivePanel::FileList => handle_file_list_keys(app, key),
        ActivePanel::OperationPanel => handle_operation_panel_keys(app, key),
    }
}

fn handle_file_list_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(selected) = app.file_list_state.selected() {
                if selected > 0 {
                    app.file_list_state.select(Some(selected - 1));
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(selected) = app.file_list_state.selected() {
                let files = app.get_current_files();
                if selected < files.len().saturating_sub(1) {
                    app.file_list_state.select(Some(selected + 1));
                }
            }
        }
        KeyCode::Char(' ') => {
            app.toggle_file_selection();
        }
        KeyCode::Char('a') => {
            app.select_all_files();
        }
        KeyCode::Char('n') => {
            app.deselect_all_files();
        }
        KeyCode::Enter => {
            app.toggle_file_selection();
        }
        _ => {}
    }
}

fn handle_operation_panel_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            match app.active_operation {
                ActiveOperation::Scan => {
                    app.account_id.push(c);
                }
                ActiveOperation::AddAppid => {
                    app.app_id_input.push(c);
                }
            }
        }
        KeyCode::Backspace => {
            match app.active_operation {
                ActiveOperation::Scan => {
                    app.account_id.pop();
                }
                ActiveOperation::AddAppid => {
                    app.app_id_input.pop();
                }
            }
        }
        KeyCode::Enter => {
            match app.active_operation {
                ActiveOperation::Scan => {
                    app.save_account_id();
                    app.execute_scan();
                }
                ActiveOperation::AddAppid => {
                    app.execute_add_appid();
                }
            }
        }
        _ => {}
    }
}

fn handle_settings_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.active_screen = ActiveScreen::Main;
        }
        KeyCode::Char('l') => {
            app.active_screen = ActiveScreen::LanguageSelection;
        }
        KeyCode::Char('d') => {
            // 切换到目录输入模式
            app.active_screen = ActiveScreen::DirectoryInput;
            app.directory_input = app.directory.as_ref()
                .map(|d| d.display().to_string())
                .unwrap_or_default();
        }
        _ => {}
    }
}

fn handle_directory_input_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.active_screen = ActiveScreen::Settings;
        }
        KeyCode::Enter => {
            app.confirm_directory_input();
        }
        KeyCode::Char(c) => {
            app.directory_input.push(c);
        }
        KeyCode::Backspace => {
            app.directory_input.pop();
        }
        _ => {}
    }
}

fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            if let Some(selected) = app.file_list_state.selected() {
                if selected > 0 {
                    app.file_list_state.select(Some(selected - 1));
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if let Some(selected) = app.file_list_state.selected() {
                let files = app.get_current_files();
                if selected < files.len().saturating_sub(1) {
                    app.file_list_state.select(Some(selected + 1));
                }
            }
        }
        MouseEventKind::Down(_) => {
            app.active_panel = ActivePanel::FileList;
        }
        _ => {}
    }
}
