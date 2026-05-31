#![forbid(unsafe_code)]

use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Target directory containing numeric .lua files.
    #[arg(short = 'd', long = "dir", global = true)]
    dir: Option<PathBuf>,

    /// 17-digit account ID. When provided, it is validated and saved locally.
    #[arg(short = 'i', long = "account-id", global = true)]
    account_id: Option<String>,

    /// Force prompting for a new account ID and save it locally.
    #[arg(long, global = true)]
    switch_account: bool,

    /// Set interface language and save it locally.
    #[arg(short = 'l', long = "language", global = true, value_enum)]
    language: Option<Language>,

    /// Force prompting for a new interface language and save it locally.
    #[arg(long, global = true)]
    switch_language: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan numeric .lua files and inject missing setStat(file_id, account_id) calls.
    Scan,
    /// Append addappid(AppID) to selected numeric .lua files.
    AddAppid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Language {
    ZhCn,
    ZhTw,
    En,
}

impl Language {
    fn code(self) -> &'static str {
        match self {
            Language::ZhCn => "zh-cn",
            Language::ZhTw => "zh-tw",
            Language::En => "en",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Language::ZhCn => "简体中文",
            Language::ZhTw => "繁體中文",
            Language::En => "English",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        match code.trim().to_ascii_lowercase().as_str() {
            "zh-cn" | "zh_cn" | "zh" | "cn" | "1" => Some(Language::ZhCn),
            "zh-tw" | "zh_tw" | "tw" | "hk" | "2" => Some(Language::ZhTw),
            "en" | "english" | "3" => Some(Language::En),
            _ => None,
        }
    }

    fn msg(self, key: Msg) -> &'static str {
        match self {
            Language::ZhCn => zh_cn(key),
            Language::ZhTw => zh_tw(key),
            Language::En => en(key),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Msg {
    ErrorPrefix,
    SelectLanguageTitle,
    SelectLanguagePrompt,
    InvalidLanguage,
    CurrentLanguage,
    CurrentAccount,
    ScanningDirectory,
    NoNumericLuaFiles,
    AllFilesReady,
    SelectionPrompt,
    RetrySuffix,
    Written,
    AlreadyExistsSkipped,
    WriteFailed,
    ScanComplete,
    AppIdPrompt,
    AppIdInvalid,
    AppIdSelectionPrompt,
    AppIdComplete,
    SavedAccountInvalid,
    AccountPrompt,
    AccountInvalid,
    DirectoryPrompt,
    DirectoryAccessFailed,
    PathIsNotDirectory,
    ReadDirFailed,
    SkipUnreadableEntry,
    SkipNonUtf8FileName,
    SkipLinkedFile,
    SkipEscapedFile,
    SkipNonRegularFile,
    ReadFailedSkipped,
    DetectFailedSkipped,
    ReadFileFailed,
    FileBoundaryFailed,
    OpenAppendFailed,
    AppendNewlineFailed,
    WriteContentFailed,
    WriteTrailingNewlineFailed,
    ConfigReadFailed,
    ConfigDirCreateFailed,
    AccountSaveFailed,
    LanguageSaveFailed,
    EmptySelection,
    InvalidToken,
    InvalidRange,
    ReversedRange,
    OutOfRange,
}

fn zh_cn(key: Msg) -> &'static str {
    match key {
        Msg::ErrorPrefix => "错误",
        Msg::SelectLanguageTitle => "请选择界面语言：",
        Msg::SelectLanguagePrompt => "请输入语言序号或代码（1/zh-cn，2/zh-tw，3/en）：",
        Msg::InvalidLanguage => "无法识别语言选项，请重新输入。",
        Msg::CurrentLanguage => "当前语言",
        Msg::CurrentAccount => "当前账号 ID",
        Msg::ScanningDirectory => "正在扫描目录",
        Msg::NoNumericLuaFiles => "未找到任何纯数字命名的 .lua 文件。",
        Msg::AllFilesReady => "所有文件都已存在对应的 setStat 属性，无需处理。",
        Msg::SelectionPrompt => {
            "请输入你想处理的文件序号（支持多种复杂格式，如 '1 2 10' 或 '1-10' 或混合输入 '1 3 5-8'）："
        }
        Msg::RetrySuffix => "请重新输入。",
        Msg::Written => "已写入",
        Msg::AlreadyExistsSkipped => "已跳过（目标语句已存在）",
        Msg::WriteFailed => "写入失败",
        Msg::ScanComplete => "完成：成功 {success} 个，跳过 {skipped} 个，失败 {failed} 个。",
        Msg::AppIdPrompt => "请输入自定义 AppID（纯数字）：",
        Msg::AppIdInvalid => "AppID 必须是非空纯数字，请重新输入。",
        Msg::AppIdSelectionPrompt => {
            "请输入要注入的文件序号或文件名（支持 '1 3 5-8'，也可直接输入 <数字文件名>.lua）："
        }
        Msg::AppIdComplete => "完成：成功 {success} 个，失败 {failed} 个。",
        Msg::SavedAccountInvalid => "本地保存的账号 ID 无效，将重新输入。",
        Msg::AccountPrompt => "请输入 17 位纯数字目标账号 ID：",
        Msg::AccountInvalid => "账号 ID 必须是 17 位纯数字字符串",
        Msg::DirectoryPrompt => "请输入包含 Lua 配置文件的本地目录路径：",
        Msg::DirectoryAccessFailed => "目录不存在或无法访问",
        Msg::PathIsNotDirectory => "路径不是目录",
        Msg::ReadDirFailed => "无法读取目录",
        Msg::SkipUnreadableEntry => "跳过无法读取的目录项",
        Msg::SkipNonUtf8FileName => "跳过非 UTF-8 文件名",
        Msg::SkipLinkedFile => "跳过链接或重解析点文件",
        Msg::SkipEscapedFile => "跳过真实路径不在目标目录内的文件",
        Msg::SkipNonRegularFile => "跳过非常规文件",
        Msg::ReadFailedSkipped => "读取失败，已跳过",
        Msg::DetectFailedSkipped => "检测失败，已跳过",
        Msg::ReadFileFailed => "无法读取文件",
        Msg::FileBoundaryFailed => "文件边界校验失败",
        Msg::OpenAppendFailed => "无法打开文件进行追加写入",
        Msg::AppendNewlineFailed => "无法补写换行符",
        Msg::WriteContentFailed => "无法写入内容",
        Msg::WriteTrailingNewlineFailed => "无法写入行尾换行",
        Msg::ConfigReadFailed => "无法读取本地配置",
        Msg::ConfigDirCreateFailed => "无法创建配置目录",
        Msg::AccountSaveFailed => "无法保存账号 ID",
        Msg::LanguageSaveFailed => "无法保存语言选项",
        Msg::EmptySelection => "输入不能为空",
        Msg::InvalidToken => "无法解析选择项",
        Msg::InvalidRange => "无法解析范围",
        Msg::ReversedRange => "范围起点不能大于终点",
        Msg::OutOfRange => "序号超出有效范围",
    }
}

fn zh_tw(key: Msg) -> &'static str {
    match key {
        Msg::ErrorPrefix => "錯誤",
        Msg::SelectLanguageTitle => "請選擇介面語言：",
        Msg::SelectLanguagePrompt => "請輸入語言序號或代碼（1/zh-cn，2/zh-tw，3/en）：",
        Msg::InvalidLanguage => "無法辨識語言選項，請重新輸入。",
        Msg::CurrentLanguage => "目前語言",
        Msg::CurrentAccount => "目前帳號 ID",
        Msg::ScanningDirectory => "正在掃描目錄",
        Msg::NoNumericLuaFiles => "未找到任何純數字命名的 .lua 檔案。",
        Msg::AllFilesReady => "所有檔案都已存在對應的 setStat 屬性，無需處理。",
        Msg::SelectionPrompt => {
            "請輸入你想處理的檔案序號（支援多種複雜格式，如 '1 2 10' 或 '1-10' 或混合輸入 '1 3 5-8'）："
        }
        Msg::RetrySuffix => "請重新輸入。",
        Msg::Written => "已寫入",
        Msg::AlreadyExistsSkipped => "已略過（目標語句已存在）",
        Msg::WriteFailed => "寫入失敗",
        Msg::ScanComplete => "完成：成功 {success} 個，略過 {skipped} 個，失敗 {failed} 個。",
        Msg::AppIdPrompt => "請輸入自訂 AppID（純數字）：",
        Msg::AppIdInvalid => "AppID 必須是非空純數字，請重新輸入。",
        Msg::AppIdSelectionPrompt => {
            "請輸入要注入的檔案序號或檔名（支援 '1 3 5-8'，也可直接輸入 <數字檔名>.lua）："
        }
        Msg::AppIdComplete => "完成：成功 {success} 個，失敗 {failed} 個。",
        Msg::SavedAccountInvalid => "本機儲存的帳號 ID 無效，將重新輸入。",
        Msg::AccountPrompt => "請輸入 17 位純數字目標帳號 ID：",
        Msg::AccountInvalid => "帳號 ID 必須是 17 位純數字字串",
        Msg::DirectoryPrompt => "請輸入包含 Lua 設定檔的本機目錄路徑：",
        Msg::DirectoryAccessFailed => "目錄不存在或無法存取",
        Msg::PathIsNotDirectory => "路徑不是目錄",
        Msg::ReadDirFailed => "無法讀取目錄",
        Msg::SkipUnreadableEntry => "略過無法讀取的目錄項目",
        Msg::SkipNonUtf8FileName => "略過非 UTF-8 檔名",
        Msg::SkipLinkedFile => "略過連結或重解析點檔案",
        Msg::SkipEscapedFile => "略過真實路徑不在目標目錄內的檔案",
        Msg::SkipNonRegularFile => "略過非常規檔案",
        Msg::ReadFailedSkipped => "讀取失敗，已略過",
        Msg::DetectFailedSkipped => "偵測失敗，已略過",
        Msg::ReadFileFailed => "無法讀取檔案",
        Msg::FileBoundaryFailed => "檔案邊界校驗失敗",
        Msg::OpenAppendFailed => "無法開啟檔案進行追加寫入",
        Msg::AppendNewlineFailed => "無法補寫換行符",
        Msg::WriteContentFailed => "無法寫入內容",
        Msg::WriteTrailingNewlineFailed => "無法寫入行尾換行",
        Msg::ConfigReadFailed => "無法讀取本機設定",
        Msg::ConfigDirCreateFailed => "無法建立設定目錄",
        Msg::AccountSaveFailed => "無法儲存帳號 ID",
        Msg::LanguageSaveFailed => "無法儲存語言選項",
        Msg::EmptySelection => "輸入不能為空",
        Msg::InvalidToken => "無法解析選擇項",
        Msg::InvalidRange => "無法解析範圍",
        Msg::ReversedRange => "範圍起點不能大於終點",
        Msg::OutOfRange => "序號超出有效範圍",
    }
}

fn en(key: Msg) -> &'static str {
    match key {
        Msg::ErrorPrefix => "Error",
        Msg::SelectLanguageTitle => "Select interface language:",
        Msg::SelectLanguagePrompt => "Enter language number or code (1/zh-cn, 2/zh-tw, 3/en):",
        Msg::InvalidLanguage => "Unrecognized language option. Please try again.",
        Msg::CurrentLanguage => "Current language",
        Msg::CurrentAccount => "Current account ID",
        Msg::ScanningDirectory => "Scanning directory",
        Msg::NoNumericLuaFiles => "No numeric .lua files were found.",
        Msg::AllFilesReady => "Every file already contains the matching setStat entry. Nothing to do.",
        Msg::SelectionPrompt => {
            "Enter file indexes to process (supports formats like '1 2 10', '1-10', or mixed input '1 3 5-8'):"
        }
        Msg::RetrySuffix => "Please try again.",
        Msg::Written => "Written",
        Msg::AlreadyExistsSkipped => "Skipped (target statement already exists)",
        Msg::WriteFailed => "Write failed",
        Msg::ScanComplete => "Done: {success} succeeded, {skipped} skipped, {failed} failed.",
        Msg::AppIdPrompt => "Enter custom AppID (digits only):",
        Msg::AppIdInvalid => "AppID must be non-empty digits. Please try again.",
        Msg::AppIdSelectionPrompt => {
            "Enter target file indexes or file names (supports '1 3 5-8', or direct names like <numeric-file>.lua):"
        }
        Msg::AppIdComplete => "Done: {success} succeeded, {failed} failed.",
        Msg::SavedAccountInvalid => "The saved account ID is invalid. Please enter it again.",
        Msg::AccountPrompt => "Enter the 17-digit target account ID:",
        Msg::AccountInvalid => "Account ID must be a 17-digit numeric string",
        Msg::DirectoryPrompt => "Enter the local directory path containing Lua config files:",
        Msg::DirectoryAccessFailed => "Directory does not exist or cannot be accessed",
        Msg::PathIsNotDirectory => "Path is not a directory",
        Msg::ReadDirFailed => "Unable to read directory",
        Msg::SkipUnreadableEntry => "Skipped unreadable directory entry",
        Msg::SkipNonUtf8FileName => "Skipped non-UTF-8 file name",
        Msg::SkipLinkedFile => "Skipped linked or reparse-point file",
        Msg::SkipEscapedFile => "Skipped file whose real path is outside the target directory",
        Msg::SkipNonRegularFile => "Skipped non-regular file",
        Msg::ReadFailedSkipped => "Read failed, skipped",
        Msg::DetectFailedSkipped => "Detection failed, skipped",
        Msg::ReadFileFailed => "Unable to read file",
        Msg::FileBoundaryFailed => "File boundary validation failed",
        Msg::OpenAppendFailed => "Unable to open file for appending",
        Msg::AppendNewlineFailed => "Unable to add separating newline",
        Msg::WriteContentFailed => "Unable to write content",
        Msg::WriteTrailingNewlineFailed => "Unable to write trailing newline",
        Msg::ConfigReadFailed => "Unable to read local config",
        Msg::ConfigDirCreateFailed => "Unable to create config directory",
        Msg::AccountSaveFailed => "Unable to save account ID",
        Msg::LanguageSaveFailed => "Unable to save language option",
        Msg::EmptySelection => "Input cannot be empty",
        Msg::InvalidToken => "Unable to parse selection item",
        Msg::InvalidRange => "Unable to parse range",
        Msg::ReversedRange => "Range start cannot be greater than range end",
        Msg::OutOfRange => "Index is outside the valid range",
    }
}

#[derive(Debug, Clone)]
struct LuaFile {
    path: PathBuf,
    root_dir: PathBuf,
    file_name: String,
    numeric_stem: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectionParseError {
    Empty,
    InvalidToken(String),
    InvalidRange(String),
    ReversedRange(String),
    OutOfRange { value: usize, max: usize },
}

impl SelectionParseError {
    fn localized(&self, language: Language) -> String {
        match self {
            SelectionParseError::Empty => language.msg(Msg::EmptySelection).to_owned(),
            SelectionParseError::InvalidToken(token) => {
                format!("{}: {token}", language.msg(Msg::InvalidToken))
            }
            SelectionParseError::InvalidRange(token) => {
                format!("{}: {token}", language.msg(Msg::InvalidRange))
            }
            SelectionParseError::ReversedRange(token) => {
                format!("{}: {token}", language.msg(Msg::ReversedRange))
            }
            SelectionParseError::OutOfRange { value, max } => {
                format!("{}: {value} (1~{max})", language.msg(Msg::OutOfRange))
            }
        }
    }
}

impl fmt::Display for SelectionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.localized(Language::ZhCn))
    }
}

impl Error for SelectionParseError {}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let cli = Cli::parse();
    let language = resolve_language(cli.language, cli.switch_language)?;
    println!(
        "{}: {}",
        language.msg(Msg::CurrentLanguage),
        language.display_name()
    );

    if let Err(err) = match cli.command.as_ref().unwrap_or(&Commands::Scan) {
        Commands::Scan => run_scan(&cli, language),
        Commands::AddAppid => run_add_appid(&cli, language),
    } {
        return Err(format!("{}: {err}", language.msg(Msg::ErrorPrefix)).into());
    }

    Ok(())
}

fn run_scan(cli: &Cli, language: Language) -> AppResult<()> {
    let account_id = resolve_account_id(cli.account_id.as_deref(), cli.switch_account, language)?;
    let dir = resolve_directory(cli.dir.as_deref(), language)?;

    println!("{}: {account_id}", language.msg(Msg::CurrentAccount));
    println!("{}: {}", language.msg(Msg::ScanningDirectory), dir.display());

    let files = scan_numeric_lua_files(&dir, language)?;
    if files.is_empty() {
        println!("{}", language.msg(Msg::NoNumericLuaFiles));
        return Ok(());
    }

    let missing = detect_missing_set_stat(&files, &account_id, language);
    if missing.is_empty() {
        println!("{}", language.msg(Msg::AllFilesReady));
        return Ok(());
    }

    print_file_menu(&missing);

    let selected = loop {
        let input = prompt(language.msg(Msg::SelectionPrompt))?;
        match parse_selection(&input, missing.len()) {
            Ok(indices) => break indices,
            Err(err) => eprintln!(
                "{}，{}",
                err.localized(language),
                language.msg(Msg::RetrySuffix)
            ),
        }
    };

    let mut success = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for index in selected {
        let file = &missing[index - 1];
        match append_set_stat(file, &account_id, language) {
            Ok(AppendResult::Written) => {
                println!("{}: {}", language.msg(Msg::Written), file.file_name);
                success += 1;
            }
            Ok(AppendResult::AlreadyExists) => {
                println!(
                    "{}: {}",
                    language.msg(Msg::AlreadyExistsSkipped),
                    file.file_name
                );
                skipped += 1;
            }
            Err(err) => {
                eprintln!("{} {}: {err}", language.msg(Msg::WriteFailed), file.file_name);
                failed += 1;
            }
        }
    }

    println!(
        "{}",
        language
            .msg(Msg::ScanComplete)
            .replace("{success}", &success.to_string())
            .replace("{skipped}", &skipped.to_string())
            .replace("{failed}", &failed.to_string())
    );
    Ok(())
}

fn run_add_appid(cli: &Cli, language: Language) -> AppResult<()> {
    let dir = resolve_directory(cli.dir.as_deref(), language)?;
    let files = scan_numeric_lua_files(&dir, language)?;
    if files.is_empty() {
        println!("{}", language.msg(Msg::NoNumericLuaFiles));
        return Ok(());
    }

    let app_id = loop {
        let input = prompt(language.msg(Msg::AppIdPrompt))?;
        if is_non_empty_digits(&input) {
            break input;
        }
        eprintln!("{}", language.msg(Msg::AppIdInvalid));
    };

    print_file_menu(&files);

    let selected = loop {
        let input = prompt(language.msg(Msg::AppIdSelectionPrompt))?;
        match parse_file_selection(&input, &files) {
            Ok(indices) => break indices,
            Err(err) => eprintln!(
                "{}，{}",
                err.localized(language),
                language.msg(Msg::RetrySuffix)
            ),
        }
    };

    let mut success = 0usize;
    let mut failed = 0usize;

    for index in selected {
        let file = &files[index - 1];
        match append_add_appid(file, &app_id, language) {
            Ok(()) => {
                println!("{}: {}", language.msg(Msg::Written), file.file_name);
                success += 1;
            }
            Err(err) => {
                eprintln!("{} {}: {err}", language.msg(Msg::WriteFailed), file.file_name);
                failed += 1;
            }
        }
    }

    println!(
        "{}",
        language
            .msg(Msg::AppIdComplete)
            .replace("{success}", &success.to_string())
            .replace("{failed}", &failed.to_string())
    );
    Ok(())
}

fn resolve_language(cli_language: Option<Language>, switch_language: bool) -> AppResult<Language> {
    if let Some(language) = cli_language {
        save_language(language)?;
        return Ok(language);
    }

    if !switch_language {
        if let Some(saved) = load_language()? {
            return Ok(saved);
        }
    }

    println!("{}", Language::ZhCn.msg(Msg::SelectLanguageTitle));
    println!("1. {}", Language::ZhCn.display_name());
    println!("2. {}", Language::ZhTw.display_name());
    println!("3. {}", Language::En.display_name());

    loop {
        let input = prompt(Language::ZhCn.msg(Msg::SelectLanguagePrompt))?;
        if let Some(language) = Language::from_code(&input) {
            save_language(language)?;
            return Ok(language);
        }
        eprintln!("{}", Language::ZhCn.msg(Msg::InvalidLanguage));
    }
}

fn resolve_account_id(
    cli_account_id: Option<&str>,
    switch_account: bool,
    language: Language,
) -> AppResult<String> {
    if let Some(account_id) = cli_account_id {
        validate_account_id(account_id, language)?;
        save_account_id(account_id, language)?;
        return Ok(account_id.to_owned());
    }

    if !switch_account {
        if let Some(saved) = load_account_id(language)? {
            if validate_account_id(&saved, language).is_ok() {
                return Ok(saved);
            }
            eprintln!("{}", language.msg(Msg::SavedAccountInvalid));
        }
    }

    loop {
        let input = prompt(language.msg(Msg::AccountPrompt))?;
        match validate_account_id(&input, language) {
            Ok(()) => {
                save_account_id(&input, language)?;
                return Ok(input);
            }
            Err(err) => eprintln!("{err}，{}", language.msg(Msg::RetrySuffix)),
        }
    }
}

fn validate_account_id(account_id: &str, language: Language) -> AppResult<()> {
    let re = Regex::new(r"^\d{17}$")?;
    if re.is_match(account_id) {
        Ok(())
    } else {
        Err(language.msg(Msg::AccountInvalid).into())
    }
}

fn resolve_directory(cli_dir: Option<&Path>, language: Language) -> AppResult<PathBuf> {
    let dir = match cli_dir {
        Some(dir) => dir.to_path_buf(),
        None => PathBuf::from(prompt(language.msg(Msg::DirectoryPrompt))?),
    };

    let canonical = fs::canonicalize(&dir).map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::DirectoryAccessFailed),
            dir.display()
        )
    })?;

    if !canonical.is_dir() {
        return Err(format!("{}: {}", language.msg(Msg::PathIsNotDirectory), canonical.display()).into());
    }

    Ok(canonical)
}

fn scan_numeric_lua_files(dir: &Path, language: Language) -> AppResult<Vec<LuaFile>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)
        .map_err(|err| format!("{} {}: {err}", language.msg(Msg::ReadDirFailed), dir.display()))?
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("{}: {err}", language.msg(Msg::SkipUnreadableEntry));
                continue;
            }
        };

        let path = entry.path();
        if !is_numeric_lua_file(&path) {
            continue;
        }

        let canonical_path = match validate_file_inside_root(&path, dir, language) {
            Ok(canonical_path) => canonical_path,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };

        if !canonical_path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name.to_owned(),
            None => {
                eprintln!("{}: {}", language.msg(Msg::SkipNonUtf8FileName), path.display());
                continue;
            }
        };

        let numeric_stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("numeric stem checked by is_numeric_lua_file")
            .to_owned();

        files.push(LuaFile {
            path: canonical_path,
            root_dir: dir.to_path_buf(),
            file_name,
            numeric_stem,
        });
    }

    files.sort_by(|left, right| {
        let left_num = left.numeric_stem.parse::<u128>();
        let right_num = right.numeric_stem.parse::<u128>();
        match (left_num, right_num) {
            (Ok(left_num), Ok(right_num)) => left_num.cmp(&right_num),
            _ => left.numeric_stem.cmp(&right.numeric_stem),
        }
    });

    Ok(files)
}

fn is_numeric_lua_file(path: &Path) -> bool {
    let extension_is_lua = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("lua"));

    let stem_is_numeric = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(is_non_empty_digits);

    extension_is_lua && stem_is_numeric
}

fn validate_file_inside_root(
    path: &Path,
    root_dir: &Path,
    language: Language,
) -> AppResult<PathBuf> {
    let metadata = fs::symlink_metadata(path).map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::FileBoundaryFailed),
            path.display()
        )
    })?;

    if is_link_or_reparse_point(&metadata) {
        return Err(format!(
            "{}: {}",
            language.msg(Msg::SkipLinkedFile),
            path.display()
        )
        .into());
    }

    if !metadata.is_file() {
        return Err(format!(
            "{}: {}",
            language.msg(Msg::SkipNonRegularFile),
            path.display()
        )
        .into());
    }

    let canonical_path = fs::canonicalize(path).map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::FileBoundaryFailed),
            path.display()
        )
    })?;

    if !canonical_path.starts_with(root_dir) {
        return Err(format!(
            "{}: {}",
            language.msg(Msg::SkipEscapedFile),
            canonical_path.display()
        )
        .into());
    }

    Ok(canonical_path)
}

fn is_link_or_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink() || has_windows_reparse_point(metadata)
}

#[cfg(windows)]
fn has_windows_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn has_windows_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

fn detect_missing_set_stat(files: &[LuaFile], account_id: &str, language: Language) -> Vec<LuaFile> {
    let mut missing = Vec::new();

    for file in files {
        let content = match fs::read_to_string(&file.path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!(
                    "{} {}: {err}",
                    language.msg(Msg::ReadFailedSkipped),
                    file.file_name
                );
                continue;
            }
        };

        match contains_set_stat(&content, &file.numeric_stem, account_id) {
            Ok(true) => {}
            Ok(false) => missing.push(file.clone()),
            Err(err) => eprintln!(
                "{} {}: {err}",
                language.msg(Msg::DetectFailedSkipped),
                file.file_name
            ),
        }
    }

    missing
}

fn contains_set_stat(content: &str, file_id: &str, account_id: &str) -> Result<bool, regex::Error> {
    let pattern = format!(
        r#"\bsetStat\s*\(\s*{}\s*,\s*"?{}"?\s*\)"#,
        regex::escape(file_id),
        regex::escape(account_id)
    );
    Regex::new(&pattern).map(|re| re.is_match(content))
}

fn print_file_menu(files: &[LuaFile]) {
    for (index, file) in files.iter().enumerate() {
        println!("{}. {}", index + 1, file.file_name);
    }
}

fn parse_selection(input: &str, max: usize) -> Result<Vec<usize>, SelectionParseError> {
    if max == 0 {
        return Err(SelectionParseError::OutOfRange { value: 1, max });
    }

    let input = input.trim();
    if input.is_empty() {
        return Err(SelectionParseError::Empty);
    }

    let mut selected = BTreeSet::new();

    for token in input.split_whitespace() {
        if token.contains('-') {
            let bounds: Vec<&str> = token.split('-').collect();
            if bounds.len() != 2 || bounds[0].is_empty() || bounds[1].is_empty() {
                return Err(SelectionParseError::InvalidRange(token.to_owned()));
            }

            let start = parse_index(bounds[0], max)?;
            let end = parse_index(bounds[1], max)?;
            if start > end {
                return Err(SelectionParseError::ReversedRange(token.to_owned()));
            }

            for value in start..=end {
                selected.insert(value);
            }
        } else {
            selected.insert(parse_index(token, max)?);
        }
    }

    if selected.is_empty() {
        Err(SelectionParseError::Empty)
    } else {
        Ok(selected.into_iter().collect())
    }
}

fn parse_file_selection(input: &str, files: &[LuaFile]) -> Result<Vec<usize>, SelectionParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(SelectionParseError::Empty);
    }

    let file_name_to_index: HashMap<String, usize> = files
        .iter()
        .enumerate()
        .map(|(index, file)| (file.file_name.to_ascii_lowercase(), index + 1))
        .collect();

    let mut selected = BTreeSet::new();

    for token in input.split_whitespace() {
        if token.ends_with(".lua") || token.ends_with(".LUA") {
            match file_name_to_index.get(&token.to_ascii_lowercase()) {
                Some(index) => {
                    selected.insert(*index);
                }
                None => return Err(SelectionParseError::InvalidToken(token.to_owned())),
            }
        } else if token.contains('-') {
            for index in parse_selection(token, files.len())? {
                selected.insert(index);
            }
        } else {
            selected.insert(parse_index(token, files.len())?);
        }
    }

    if selected.is_empty() {
        Err(SelectionParseError::Empty)
    } else {
        Ok(selected.into_iter().collect())
    }
}

fn parse_index(token: &str, max: usize) -> Result<usize, SelectionParseError> {
    if !is_non_empty_digits(token) {
        return Err(SelectionParseError::InvalidToken(token.to_owned()));
    }

    let value = token
        .parse::<usize>()
        .map_err(|_| SelectionParseError::InvalidToken(token.to_owned()))?;

    if value == 0 || value > max {
        Err(SelectionParseError::OutOfRange { value, max })
    } else {
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppendResult {
    Written,
    AlreadyExists,
}

fn append_set_stat(
    file: &LuaFile,
    account_id: &str,
    language: Language,
) -> AppResult<AppendResult> {
    let checked_path = validate_file_inside_root(&file.path, &file.root_dir, language)?;
    let content = fs::read_to_string(&checked_path).map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::ReadFileFailed),
            checked_path.display()
        )
    })?;

    if contains_set_stat(&content, &file.numeric_stem, account_id)? {
        return Ok(AppendResult::AlreadyExists);
    }

    let line = format!(r#"setStat({}, "{}")"#, file.numeric_stem, account_id);
    append_line(&checked_path, &file.root_dir, &content, &line, language)?;
    Ok(AppendResult::Written)
}

fn append_add_appid(file: &LuaFile, app_id: &str, language: Language) -> AppResult<()> {
    let checked_path = validate_file_inside_root(&file.path, &file.root_dir, language)?;
    let content = fs::read_to_string(&checked_path).map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::ReadFileFailed),
            checked_path.display()
        )
    })?;
    let line = format!("addappid({app_id})");
    append_line(&checked_path, &file.root_dir, &content, &line, language)
}

fn append_line(
    path: &Path,
    root_dir: &Path,
    current_content: &str,
    line: &str,
    language: Language,
) -> AppResult<()> {
    let checked_path = validate_file_inside_root(path, root_dir, language)?;
    let mut options = OpenOptions::new();
    options.append(true);
    harden_open_options(&mut options);

    let mut file = options
        .open(&checked_path)
        .map_err(|err| format!("{} {}: {err}", language.msg(Msg::OpenAppendFailed), checked_path.display()))?;

    let opened_metadata = file.metadata().map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::FileBoundaryFailed),
            checked_path.display()
        )
    })?;

    if is_link_or_reparse_point(&opened_metadata) || !opened_metadata.is_file() {
        return Err(format!(
            "{}: {}",
            language.msg(Msg::SkipLinkedFile),
            checked_path.display()
        )
        .into());
    }

    if !current_content.is_empty() && !current_content.ends_with('\n') {
        file.write_all(b"\n").map_err(|err| {
            format!(
                "{} {}: {err}",
                language.msg(Msg::AppendNewlineFailed),
                checked_path.display()
            )
        })?;
    }

    file.write_all(line.as_bytes()).map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::WriteContentFailed),
            checked_path.display()
        )
    })?;
    file.write_all(b"\n").map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::WriteTrailingNewlineFailed),
            checked_path.display()
        )
    })?;
    Ok(())
}

#[cfg(windows)]
fn harden_open_options(options: &mut OpenOptions) {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
}

#[cfg(not(windows))]
fn harden_open_options(_options: &mut OpenOptions) {}

fn prompt(message: &str) -> AppResult<String> {
    print!("{message} ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_owned())
}

fn is_non_empty_digits(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}

fn config_dir() -> AppResult<PathBuf> {
    let base_dir = if let Ok(app_data) = env::var("APPDATA") {
        PathBuf::from(app_data).join("faos-cli")
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".faos-cli")
    } else if let Ok(user_profile) = env::var("USERPROFILE") {
        PathBuf::from(user_profile).join(".faos-cli")
    } else {
        env::current_dir()?.join(".faos-cli")
    };

    Ok(base_dir)
}

fn account_config_file_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("account_id.txt"))
}

fn language_config_file_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("language.txt"))
}

fn load_account_id(language: Language) -> AppResult<Option<String>> {
    let path = account_config_file_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let value = fs::read_to_string(&path)
        .map_err(|err| format!("{} {}: {err}", language.msg(Msg::ConfigReadFailed), path.display()))?;
    Ok(Some(value.trim().to_owned()))
}

fn save_account_id(account_id: &str, language: Language) -> AppResult<()> {
    let path = account_config_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "{} {}: {err}",
                language.msg(Msg::ConfigDirCreateFailed),
                parent.display()
            )
        })?;
    }

    fs::write(&path, account_id)
        .map_err(|err| format!("{} {}: {err}", language.msg(Msg::AccountSaveFailed), path.display()))?;
    Ok(())
}

fn load_language() -> AppResult<Option<Language>> {
    let path = language_config_file_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let value = fs::read_to_string(&path)?;
    Ok(Language::from_code(&value))
}

fn save_language(language: Language) -> AppResult<()> {
    let path = language_config_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "{} {}: {err}",
                Language::ZhCn.msg(Msg::ConfigDirCreateFailed),
                parent.display()
            )
        })?;
    }

    fs::write(&path, language.code()).map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::LanguageSaveFailed),
            path.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_space_separated_indices() {
        assert_eq!(parse_selection("1 2 10", 10).unwrap(), vec![1, 2, 10]);
    }

    #[test]
    fn parses_inclusive_ranges() {
        assert_eq!(parse_selection("1-5", 10).unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn parses_mixed_input_and_deduplicates() {
        assert_eq!(
            parse_selection("1 3 5-8 8 10", 10).unwrap(),
            vec![1, 3, 5, 6, 7, 8, 10]
        );
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(parse_selection("   ", 10), Err(SelectionParseError::Empty));
    }

    #[test]
    fn rejects_zero_and_out_of_range() {
        assert_eq!(
            parse_selection("0", 10),
            Err(SelectionParseError::OutOfRange { value: 0, max: 10 })
        );
        assert_eq!(
            parse_selection("11", 10),
            Err(SelectionParseError::OutOfRange { value: 11, max: 10 })
        );
    }

    #[test]
    fn rejects_invalid_ranges() {
        assert_eq!(
            parse_selection("1-", 10),
            Err(SelectionParseError::InvalidRange("1-".to_owned()))
        );
        assert_eq!(
            parse_selection("8-5", 10),
            Err(SelectionParseError::ReversedRange("8-5".to_owned()))
        );
    }

    #[test]
    fn matches_set_stat_with_optional_account_quotes_and_spaces() {
        let file_id = "0".repeat(7);
        let account_id = "0".repeat(17);
        let content = format!(
            r#"
            setStat({file_id}, "{account_id}")
            setStat(42, {account_id})
        "#
        );

        assert!(contains_set_stat(&content, &file_id, &account_id).unwrap());
        assert!(contains_set_stat(&content, "42", &account_id).unwrap());
        assert!(!contains_set_stat(&content, "99", &account_id).unwrap());
    }

    #[test]
    fn parses_file_selection_with_ranges_and_file_names() {
        let files = vec![
            LuaFile {
                path: PathBuf::from("100.lua"),
                root_dir: PathBuf::from("."),
                file_name: "100.lua".to_owned(),
                numeric_stem: "100".to_owned(),
            },
            LuaFile {
                path: PathBuf::from("200.LUA"),
                root_dir: PathBuf::from("."),
                file_name: "200.LUA".to_owned(),
                numeric_stem: "200".to_owned(),
            },
            LuaFile {
                path: PathBuf::from("300.lua"),
                root_dir: PathBuf::from("."),
                file_name: "300.lua".to_owned(),
                numeric_stem: "300".to_owned(),
            },
        ];

        assert_eq!(parse_file_selection("1 200.lua 2-3", &files).unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn parses_language_aliases() {
        assert_eq!(Language::from_code("zh-cn"), Some(Language::ZhCn));
        assert_eq!(Language::from_code("2"), Some(Language::ZhTw));
        assert_eq!(Language::from_code("en"), Some(Language::En));
        assert_eq!(Language::from_code("unknown"), None);
    }

    #[test]
    fn validates_file_must_stay_inside_root() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = env::temp_dir().join(format!(
            "faos-cli-boundary-test-{}-{unique}",
            std::process::id()
        ));
        let root = base.join("root");
        let outside = base.join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let inside_file = root.join("100.lua");
        let outside_file = outside.join("200.lua");
        fs::write(&inside_file, "-- inside").unwrap();
        fs::write(&outside_file, "-- outside").unwrap();

        let canonical_root = fs::canonicalize(&root).unwrap();
        assert!(validate_file_inside_root(&inside_file, &canonical_root, Language::En).is_ok());
        assert!(validate_file_inside_root(&outside_file, &canonical_root, Language::En).is_err());

        let _ = fs::remove_dir_all(base);
    }
}
