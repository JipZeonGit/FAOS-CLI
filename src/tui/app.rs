use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use crate::core::config::*;
use crate::core::i18n::*;
use crate::core::lua::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveScreen {
    LanguageSelection,
    Main,
    Settings,
    DirectoryInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivePanel {
    FileList,
    OperationPanel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveOperation {
    Scan,
    AddAppid,
}

pub struct App {
    pub running: bool,
    pub active_screen: ActiveScreen,
    pub active_panel: ActivePanel,
    pub active_operation: ActiveOperation,
    pub language: Option<Language>,
    pub account_id: String,
    pub directory: Option<PathBuf>,
    // 全部lua文件
    pub all_files: Vec<LuaFile>,
    // 未写入setStat的文件（用于Scan）
    pub scan_files: Vec<LuaFile>,
    pub selected_files: Vec<bool>,
    pub file_list_state: ratatui::widgets::ListState,
    pub app_id_input: String,
    pub status_message: String,
    pub status_time: Option<Instant>,
    pub language_list_state: ratatui::widgets::ListState,
    // 是否已加载文件
    pub files_loaded: bool,
    // 目录输入
    pub directory_input: String,
}

impl App {
    pub fn new() -> Self {
        let mut language_list_state = ratatui::widgets::ListState::default();
        language_list_state.select(Some(0));

        let mut file_list_state = ratatui::widgets::ListState::default();
        file_list_state.select(Some(0));

        Self {
            running: true,
            active_screen: ActiveScreen::LanguageSelection,
            active_panel: ActivePanel::FileList,
            active_operation: ActiveOperation::Scan,
            language: None,
            account_id: String::new(),
            directory: None,
            all_files: Vec::new(),
            scan_files: Vec::new(),
            selected_files: Vec::new(),
            file_list_state,
            app_id_input: String::new(),
            status_message: String::new(),
            status_time: None,
            language_list_state,
            files_loaded: false,
            directory_input: String::new(),
        }
    }

    pub fn init(&mut self) {
        if let Ok(Some(lang)) = load_language() {
            self.language = Some(lang);
            self.active_screen = ActiveScreen::Main;
            self.load_saved_config();
            // 自动加载文件
            self.load_files();
        }
    }

    pub fn load_saved_config(&mut self) {
        if let Ok(Some(dir)) = load_dir() {
            let path = PathBuf::from(dir);
            // 规范化路径以匹配 fs::canonicalize 格式
            self.directory = Some(fs::canonicalize(&path).unwrap_or(path));
        }

        if let Some(lang) = self.language {
            if let Ok(Some(account)) = load_account_id(lang) {
                self.account_id = account;
            }
        }
    }

    pub fn select_language(&mut self, language: Language) {
        self.language = Some(language);
        let _ = save_language(language);
        self.active_screen = ActiveScreen::Main;
        self.load_saved_config();
        // 自动加载文件
        self.load_files();
        self.set_status(language.msg(Msg::CurrentLanguage).to_string() + ": " + language.display_name());
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
        self.status_time = Some(Instant::now());
    }

    /// 获取当前操作对应的文件列表
    pub fn get_current_files(&self) -> &Vec<LuaFile> {
        match self.active_operation {
            ActiveOperation::Scan => &self.scan_files,
            ActiveOperation::AddAppid => &self.all_files,
        }
    }

    /// 加载文件
    pub fn load_files(&mut self) {
        let lang = match self.language {
            Some(lang) => lang,
            None => return,
        };

        let dir = match &self.directory {
            Some(dir) => dir.clone(),
            None => {
                self.set_status(lang.msg(Msg::NoDirectorySelected).to_string());
                return;
            }
        };

        // 加载全部lua文件
        match scan_numeric_lua_files(&dir, lang) {
            Ok(files) => {
                let total_count = files.len();
                self.all_files = files;
                
                // 筛选未写入setStat的文件
                if !self.account_id.is_empty() {
                    self.scan_files = detect_missing_set_stat(&self.all_files, &self.account_id, lang);
                } else {
                    self.scan_files = self.all_files.clone();
                }
                
                self.files_loaded = true;
                
                // 根据当前操作更新显示的文件
                self.update_file_list_for_operation();
                
                let scan_count = self.scan_files.len();
                
                if self.active_operation == ActiveOperation::Scan && scan_count == 0 && total_count > 0 {
                    self.set_status(lang.msg(Msg::AllFilesReady).to_string());
                } else {
                    self.set_status(format!("{}: {} / {}: {}", 
                        lang.msg(Msg::FilesLabel), total_count,
                        lang.msg(Msg::ScanSettings), scan_count));
                }
            }
            Err(err) => {
                self.set_status(format!("Error: {}", err));
            }
        }
    }

    /// 根据当前操作更新文件列表显示
    pub fn update_file_list_for_operation(&mut self) {
        let files = match self.active_operation {
            ActiveOperation::Scan => &self.scan_files,
            ActiveOperation::AddAppid => &self.all_files,
        };
        
        self.selected_files = vec![false; files.len()];
        // 始终重置为新的 ListState，清除旧的 offset/scroll 状态
        self.file_list_state = ratatui::widgets::ListState::default();
        if !files.is_empty() {
            self.file_list_state.select(Some(0));
        }
    }

    /// 切换操作模式
    pub fn switch_operation(&mut self, operation: ActiveOperation) {
        self.active_operation = operation;
        self.active_panel = ActivePanel::OperationPanel;
        self.app_id_input.clear();
        self.update_file_list_for_operation();
    }

    pub fn toggle_file_selection(&mut self) {
        if let Some(index) = self.file_list_state.selected() {
            if index < self.selected_files.len() {
                self.selected_files[index] = !self.selected_files[index];
            }
        }
    }

    pub fn select_all_files(&mut self) {
        for selected in &mut self.selected_files {
            *selected = true;
        }
    }

    pub fn deselect_all_files(&mut self) {
        for selected in &mut self.selected_files {
            *selected = false;
        }
    }

    pub fn get_selected_files(&self) -> Vec<&LuaFile> {
        let files = match self.active_operation {
            ActiveOperation::Scan => &self.scan_files,
            ActiveOperation::AddAppid => &self.all_files,
        };
        
        files
            .iter()
            .zip(self.selected_files.iter())
            .filter(|(_, selected)| **selected)
            .map(|(file, _)| file)
            .collect()
    }

    pub fn execute_scan(&mut self) {
        let lang = match self.language {
            Some(lang) => lang,
            None => return,
        };

        if self.account_id.is_empty() {
            self.set_status(lang.msg(Msg::AccountPrompt).to_string());
            return;
        }

        if let Err(err) = validate_account_id(&self.account_id, lang) {
            self.set_status(err.to_string());
            return;
        }

        // 如果scan_files为空，提示全部已存在
        if self.scan_files.is_empty() {
            self.set_status(lang.msg(Msg::AllFilesReady).to_string());
            return;
        }

        let selected_files = self.get_selected_files();
        if selected_files.is_empty() {
            self.set_status(lang.msg(Msg::EmptySelection).to_string());
            return;
        }

        let mut success = 0;
        let mut skipped = 0;
        let mut failed = 0;

        for file in selected_files {
            match append_set_stat(file, &self.account_id, lang) {
                Ok(AppendResult::Written) => {
                    success += 1;
                }
                Ok(AppendResult::AlreadyExists) => {
                    skipped += 1;
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        let msg = lang
            .msg(Msg::ScanComplete)
            .replace("{success}", &success.to_string())
            .replace("{skipped}", &skipped.to_string())
            .replace("{failed}", &failed.to_string());
        self.set_status(msg);
        
        // 重新加载文件，更新scan_files
        self.load_files();
    }

    pub fn execute_add_appid(&mut self) {
        let lang = match self.language {
            Some(lang) => lang,
            None => return,
        };

        if self.app_id_input.is_empty() {
            self.set_status(lang.msg(Msg::AppIdPrompt).to_string());
            return;
        }

        if !is_non_empty_digits(&self.app_id_input) {
            self.set_status(lang.msg(Msg::AppIdInvalid).to_string());
            return;
        }

        let selected_files = self.get_selected_files();
        if selected_files.is_empty() {
            self.set_status(lang.msg(Msg::EmptySelection).to_string());
            return;
        }

        let mut success = 0;
        let mut failed = 0;

        for file in selected_files {
            match append_add_appid(file, &self.app_id_input, lang) {
                Ok(()) => {
                    success += 1;
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        let msg = lang
            .msg(Msg::AppIdComplete)
            .replace("{success}", &success.to_string())
            .replace("{failed}", &failed.to_string());
        self.set_status(msg);
    }

    pub fn save_account_id(&mut self) {
        if let Some(lang) = self.language {
            let _ = save_account_id(&self.account_id, lang);
            self.set_status(lang.msg(Msg::AccountSaveFailed).to_string());
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    /// 确认目录输入
    pub fn confirm_directory_input(&mut self) {
        let path = PathBuf::from(&self.directory_input);
        if path.exists() && path.is_dir() {
            // 规范化路径，确保与 fs::canonicalize 的文件路径格式一致（Windows \\?\ 前缀）
            match fs::canonicalize(&path) {
                Ok(canonical) => {
                    self.directory = Some(canonical.clone());
                    if let Some(lang) = self.language {
                        let _ = save_dir(&canonical.to_string_lossy(), lang);
                    }
                    self.load_files();
                    self.active_screen = ActiveScreen::Main;
                    self.set_status(format!("{}: {}", 
                        self.language.map(|l| l.msg(Msg::CurrentDirectory)).unwrap_or("Directory"),
                        strip_unc_prefix(&canonical.display().to_string())));
                }
                Err(err) => {
                    let lang = self.language.unwrap_or(crate::core::i18n::Language::En);
                    self.set_status(format!("{}: {} - {}", 
                        lang.msg(Msg::DirectoryAccessFailed), path.display(), err));
                }
            }
        } else {
            let lang = self.language.unwrap_or(crate::core::i18n::Language::En);
            self.set_status(lang.msg(Msg::DirectoryAccessFailed).to_string());
        }
    }
}
