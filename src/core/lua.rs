use regex::Regex;
use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::i18n::{Language, Msg};

pub type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub struct LuaFile {
    pub path: PathBuf,
    pub root_dir: PathBuf,
    pub file_name: String,
    pub numeric_stem: String,
}

#[derive(Debug, Clone)]
pub struct SelectedDirectory {
    pub canonical: PathBuf,
    pub display: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionParseError {
    Empty,
    InvalidToken(String),
    InvalidRange(String),
    ReversedRange(String),
    OutOfRange { value: usize, max: usize },
}

impl SelectionParseError {
    pub fn localized(&self, language: Language) -> String {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendResult {
    Written,
    AlreadyExists,
}

pub fn scan_numeric_lua_files(dir: &Path, language: Language) -> AppResult<Vec<LuaFile>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)
        .map_err(|err| format!("{} {}: {err}", language.msg(Msg::ReadDirFailed), dir.display()))?
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                continue;
            }
        };

        let path = entry.path();
        if !is_numeric_lua_file(&path) {
            continue;
        }

        let canonical_path = match validate_file_inside_root(&path, dir, language) {
            Ok(canonical_path) => canonical_path,
            Err(_) => {
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

        let numeric_stem = match path.file_stem().and_then(|stem| stem.to_str()) {
            Some(stem) => stem.to_owned(),
            None => {
                eprintln!("{}: {}", language.msg(Msg::SkipNonUtf8FileName), path.display());
                continue;
            }
        };

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

pub fn is_numeric_lua_file(path: &Path) -> bool {
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

pub fn validate_file_inside_root(
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

pub fn detect_missing_set_stat(files: &[LuaFile], account_id: &str, _language: Language) -> Vec<LuaFile> {
    let mut missing = Vec::new();

    for file in files {
        let content = match fs::read_to_string(&file.path) {
            Ok(content) => content,
            Err(_) => {
                continue;
            }
        };

        match contains_set_stat(&content, &file.numeric_stem, account_id) {
            Ok(true) => {}
            Ok(false) => missing.push(file.clone()),
            Err(_) => {}
        }
    }

    missing
}

pub fn contains_set_stat(content: &str, file_id: &str, account_id: &str) -> Result<bool, regex::Error> {
    let pattern = format!(
        r#"\bsetStat\s*\(\s*{}\s*,\s*"?{}"?\s*\)"#,
        regex::escape(file_id),
        regex::escape(account_id)
    );
    Regex::new(&pattern).map(|re| re.is_match(content))
}

pub fn print_file_menu(files: &[LuaFile]) {
    for (index, file) in files.iter().enumerate() {
        println!("{}. {}", index + 1, file.file_name);
    }
}

pub fn parse_selection(input: &str, max: usize) -> Result<Vec<usize>, SelectionParseError> {
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

pub fn parse_file_selection(input: &str, files: &[LuaFile]) -> Result<Vec<usize>, SelectionParseError> {
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

pub fn append_set_stat(
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

pub fn append_add_appid(file: &LuaFile, app_id: &str, language: Language) -> AppResult<()> {
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

pub fn is_non_empty_digits(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}

pub fn validate_account_id(account_id: &str, language: Language) -> AppResult<()> {
    let re = Regex::new(r"^\d{17}$")?;
    if re.is_match(account_id) {
        Ok(())
    } else {
        Err(language.msg(Msg::AccountInvalid).into())
    }
}

/// 移除Windows长路径前缀 `\\?\` 用于显示
pub fn strip_unc_prefix(path: &str) -> &str {
    path.strip_prefix(r"\\?\").unwrap_or(path)
}
