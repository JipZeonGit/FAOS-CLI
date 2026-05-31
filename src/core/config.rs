use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use super::i18n::{Language, Msg};

pub type AppResult<T> = Result<T, Box<dyn Error>>;

pub fn config_dir() -> AppResult<PathBuf> {
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

pub fn account_config_file_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("account_id.txt"))
}

pub fn language_config_file_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("language.txt"))
}

pub fn dir_config_file_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("dir.txt"))
}

pub fn load_account_id(language: Language) -> AppResult<Option<String>> {
    let path = account_config_file_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let value = fs::read_to_string(&path)
        .map_err(|err| format!("{} {}: {err}", language.msg(Msg::ConfigReadFailed), path.display()))?;
    Ok(Some(value.trim().to_owned()))
}

pub fn save_account_id(account_id: &str, language: Language) -> AppResult<()> {
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

pub fn load_language() -> AppResult<Option<Language>> {
    let path = language_config_file_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let value = fs::read_to_string(&path)?;
    Ok(Language::from_code(&value))
}

pub fn save_language(language: Language) -> AppResult<()> {
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

pub fn load_dir() -> AppResult<Option<String>> {
    let path = dir_config_file_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let value = fs::read_to_string(&path)?;
    Ok(Some(value.trim().to_owned()))
}

pub fn save_dir(dir: &str, language: Language) -> AppResult<()> {
    let path = dir_config_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "{} {}: {err}",
                language.msg(Msg::ConfigDirCreateFailed),
                parent.display()
            )
        })?;
    }

    fs::write(&path, dir).map_err(|err| {
        format!(
            "{} {}: {err}",
            language.msg(Msg::DirSaveFailed),
            path.display()
        )
    })?;
    Ok(())
}
