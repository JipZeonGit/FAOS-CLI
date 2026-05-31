use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::core::config::{load_account_id, load_dir, load_language, save_account_id, save_dir, save_language};
use crate::core::i18n::{Language, Msg};
use crate::core::lua::{*, strip_unc_prefix};

pub fn resolve_language(cli_language: Option<Language>, switch_language: bool) -> AppResult<Language> {
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

pub fn resolve_account_id(
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

pub fn resolve_directory(cli_dir: Option<&Path>, language: Language) -> AppResult<SelectedDirectory> {
    let dir = match cli_dir {
        Some(dir) => dir.to_path_buf(),
        None => {
            if let Some(saved) = load_dir()? {
                PathBuf::from(saved)
            } else {
                PathBuf::from(prompt(language.msg(Msg::DirPrompt))?)
            }
        }
    };
    let display = dir.clone();

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

    save_dir(&canonical.to_string_lossy(), language)?;

    Ok(SelectedDirectory { canonical, display })
}

pub fn run_scan(
    dir: Option<&Path>,
    account_id: Option<&str>,
    switch_account: bool,
    language: Language,
) -> AppResult<()> {
    let account_id = resolve_account_id(account_id, switch_account, language)?;
    let dir = resolve_directory(dir, language)?;

    println!("{}: {account_id}", language.msg(Msg::CurrentAccount));
    println!(
        "{}: {}",
        language.msg(Msg::ScanningDirectory),
        strip_unc_prefix(&dir.display.display().to_string())
    );

    let files = scan_numeric_lua_files(&dir.canonical, language)?;
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

pub fn run_add_appid(
    dir: Option<&Path>,
    language: Language,
) -> AppResult<()> {
    let dir = resolve_directory(dir, language)?;
    let files = scan_numeric_lua_files(&dir.canonical, language)?;
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

pub fn run_switch_dir(dir: Option<&Path>, language: Language) -> AppResult<()> {
    let dir = resolve_directory(dir, language)?;
    println!(
        "{}: {}",
        language.msg(Msg::CurrentDirectory),
        strip_unc_prefix(&dir.canonical.display().to_string())
    );
    Ok(())
}

fn prompt(message: &str) -> AppResult<String> {
    print!("{message} ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_owned())
}
