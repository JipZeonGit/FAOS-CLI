#![forbid(unsafe_code)]

mod cli;
mod core;
mod tui;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::core::i18n::Language;

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

    /// Use CLI mode instead of TUI
    #[arg(long, global = true)]
    cli: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch TUI interface (default)
    Tui,
    /// Scan numeric .lua files and inject missing setStat(file_id, account_id) calls.
    Scan,
    /// Append addappid(AppID) to selected numeric .lua files.
    AddAppid,
    /// Switch the target directory and save it locally.
    SwitchDir,
}

fn main() {
    let cli = Cli::parse();

    let use_tui = !cli.cli && cli.command.is_none();

    if use_tui {
        if let Err(err) = tui::run_tui() {
            eprintln!("TUI Error: {}", err);
            std::process::exit(1);
        }
    } else {
        if let Err(err) = run_cli(&cli) {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn run_cli(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let language = cli::resolve_language(cli.language, cli.switch_language)?;

    match cli.command.as_ref().unwrap_or(&Commands::Scan) {
        Commands::Tui => {
            tui::run_tui()?;
        }
        Commands::Scan => {
            cli::run_scan(
                cli.dir.as_deref(),
                cli.account_id.as_deref(),
                cli.switch_account,
                language,
            )?;
        }
        Commands::AddAppid => {
            cli::run_add_appid(cli.dir.as_deref(), language)?;
        }
        Commands::SwitchDir => {
            cli::run_switch_dir(cli.dir.as_deref(), language)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::core::lua::*;

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
        use std::path::PathBuf;

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
        use super::core::i18n::Language;

        assert_eq!(Language::from_code("zh-cn"), Some(Language::ZhCn));
        assert_eq!(Language::from_code("2"), Some(Language::ZhTw));
        assert_eq!(Language::from_code("en"), Some(Language::En));
        assert_eq!(Language::from_code("unknown"), None);
    }

    #[test]
    fn validates_file_must_stay_inside_root() {
        use std::env;
        use std::fs;
        use super::core::i18n::Language;

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
