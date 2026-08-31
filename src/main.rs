mod app_config;
mod app_paths;
mod clear;
mod document_id;
mod downloader;
mod getter;
mod searcher;
mod status;
mod store;
mod submission_year;
mod updater;
mod zip_archive;

use clap::{Command, CommandFactory, FromArgMatches, Parser};

const HELP_TEMPLATE: &str =
    "{before-help}{about-with-newline}\n使い方: {usage}\n\n{all-args}{after-help}";

#[derive(Parser, Debug)]
#[command(
    name = "edinet",
    version = env!("CARGO_PKG_VERSION"),
    about = "EDINET の有価証券報告書を取得・解析するCLIツール",
    arg_required_else_help = true,
    after_help = "使用例:\n  \
                  edinet setup --key YOUR_API_KEY\n  \
                  edinet update\n  \
                  edinet search ストライク --limit 3\n  \
                  edinet get --company ストライク\n  \
                  edinet download pdf ./downloads --company ストライク\n\n\
                  各コマンドの詳細は `edinet <COMMAND> --help` で確認できます。"
)]
struct Cli {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(clap::Subcommand, Debug)]
enum SubCommand {
    #[command(
        about = "EDINET API の API キーを登録します",
        after_help = "使用例:\n  edinet setup --key YOUR_API_KEY",
        visible_alias = "init"
    )]
    Setup(crate::app_config::SetupArgs),
    #[command(
        about = "検索インデックスを EDINET から更新します",
        after_help = "使用例:\n  \
                      edinet update\n  \
                      edinet update --years 3 --concurrency 8\n  \
                      edinet update --years 3 --sequential\n  \
                      edinet update --today\n  \
                      edinet update --from 2026-04-01 --to 2026-04-30",
        visible_alias = "u"
    )]
    Update(crate::updater::command::UpdateArgs),
    #[command(
        about = "有価証券報告書を検索します",
        after_help = "使用例:\n  \
                      edinet search ストライク\n  \
                      edinet search --sec-code 7203\n  \
                      edinet search --company ストライク --year 2025\n  \
                      edinet search トヨタ --from 2026-04-01 --to 2026-04-30 --limit 20\n  \
                      edinet search --company ストライク --json",
        visible_alias = "s"
    )]
    Search(crate::searcher::command::SearchArgs),
    #[command(
        about = "有価証券報告書を取得し、内容を項目別に出力します",
        after_help = "使用例:\n  \
                      edinet get --company ストライク\n  \
                      edinet get --edinet-code E32380 --lang ja\n  \
                      edinet get --company ストライク --year 2025\n  \
                      edinet get --doc-id S100XAN1\n  \
                      edinet get --company ストライク --offline",
        visible_alias = "g"
    )]
    Get(crate::getter::command::GetArgs),
    #[command(
        about = "有価証券報告書を PDF などの形式でダウンロードします",
        after_help = "使用例:\n  \
                      edinet download pdf ./downloads --company ストライク\n  \
                      edinet download csv ./downloads --doc-id S100XAN1 --extract\n  \
                      edinet download pdf ./downloads --company ストライク --year 2025\n  \
                      edinet download xbrl ./downloads --edinet-code E32380",
        visible_alias = "d"
    )]
    Download(crate::downloader::command::DownloadArgs),
    #[command(
        about = "検索インデックスとダウンロード済み CSV キャッシュを削除します",
        after_help = "使用例:\n  edinet clear",
        visible_alias = "c"
    )]
    Clear,
    #[command(
        about = "各種状態を表示します",
        after_help = "使用例:\n  edinet status",
        visible_alias = "st"
    )]
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let matches = cli_command().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;

    match cli.subcommand {
        SubCommand::Setup(command) => crate::app_config::run_setup(command)?,
        SubCommand::Update(command) => crate::updater::command::run(command).await?,
        SubCommand::Search(command) => crate::searcher::command::run(command).await?,
        SubCommand::Get(command) => crate::getter::command::run(command).await?,
        SubCommand::Download(command) => crate::downloader::command::run(command).await?,
        SubCommand::Clear => {
            crate::clear::clear_local_data()?;
        }
        SubCommand::Status => {
            crate::status::run().await?;
        }
    }

    Ok(())
}

fn cli_command() -> Command {
    let mut command = Cli::command();
    command.build();
    localize_help(command)
}

fn localize_help(command: Command) -> Command {
    command
        .help_template(HELP_TEMPLATE)
        .subcommand_help_heading("コマンド")
        .mut_args(|arg| {
            let heading = if arg.get_index().is_some() {
                "引数"
            } else {
                "オプション"
            };
            let arg = arg.help_heading(heading);

            match arg.get_id().as_str() {
                "help" => arg.help("ヘルプを表示します"),
                "version" => arg.help("バージョンを表示します"),
                _ => arg,
            }
        })
        .mut_subcommands(|subcommand| {
            let subcommand = if subcommand.get_name() == "help" {
                subcommand.about("指定したコマンドのヘルプを表示します")
            } else {
                subcommand
            };
            localize_help(subcommand)
        })
}

#[cfg(test)]
mod tests {
    use super::cli_command;

    fn render_help(path: &[&str]) -> String {
        let mut command = cli_command();
        let mut current = &mut command;

        for name in path {
            current = current
                .find_subcommand_mut(name)
                .unwrap_or_else(|| panic!("subcommand `{name}` must exist"));
        }

        let mut output = Vec::new();
        current
            .write_help(&mut output)
            .expect("help must be renderable");
        String::from_utf8(output).expect("help must be UTF-8")
    }

    fn help_output(args: &[&str]) -> String {
        cli_command()
            .try_get_matches_from(args)
            .expect_err("help flag must stop argument parsing")
            .to_string()
    }

    #[test]
    fn document_commands_reject_unsafe_document_ids() {
        for args in [
            &["edinet", "get", "--doc-id", "../file1"][..],
            &[
                "edinet",
                "download",
                "csv",
                "./downloads",
                "--doc-id",
                "S100/AN1",
            ][..],
        ] {
            let error = cli_command()
                .try_get_matches_from(args)
                .expect_err("unsafe document ID must be rejected");
            assert!(
                error
                    .to_string()
                    .contains("document ID must be exactly 8 uppercase ASCII letters or digits")
            );
        }
    }

    #[test]
    fn root_help_guides_initial_setup_and_command_discovery() {
        let help = render_help(&[]);

        assert!(help.contains("edinet setup --key YOUR_API_KEY"));
        assert!(help.contains("edinet update"));
        assert!(help.contains("edinet <COMMAND> --help"));
        assert!(help.contains("EDINET の有価証券報告書を取得・解析するCLIツール"));
        assert!(help.contains("使い方: edinet <COMMAND>"));
        assert!(help.contains("コマンド:"));
        assert!(help.contains("オプション:"));
        for alias in [
            "[aliases: init]",
            "[aliases: u]",
            "[aliases: s]",
            "[aliases: g]",
            "[aliases: d]",
            "[aliases: c]",
            "[aliases: st]",
        ] {
            assert!(help.contains(alias), "root help must contain `{alias}`");
        }
        assert!(!help.contains("Print help"));
    }

    #[test]
    fn every_command_has_a_usage_example() {
        for (path, example) in [
            (&["setup"][..], "edinet setup --key YOUR_API_KEY"),
            (&["update"][..], "edinet update --years 3"),
            (&["search"][..], "edinet search --sec-code 7203"),
            (&["get"][..], "edinet get --company ストライク"),
            (
                &["download"][..],
                "edinet download csv ./downloads --doc-id S100XAN1 --extract",
            ),
            (&["clear"][..], "edinet clear"),
            (&["status"][..], "edinet status"),
        ] {
            let help = render_help(path);
            assert!(
                help.contains(example),
                "help for `{}` must contain `{example}`",
                path.join(" ")
            );
        }
    }

    #[test]
    fn destructive_and_sensitive_commands_explain_their_scope() {
        assert!(render_help(&["clear"]).contains("CSV キャッシュを削除します"));
        assert!(render_help(&["status"]).contains("各種状態を表示します"));
    }

    #[test]
    fn update_years_requires_positive_value_and_excludes_other_periods() {
        assert!(
            cli_command()
                .try_get_matches_from(["edinet", "update", "--years", "0"])
                .is_err()
        );
        assert!(
            cli_command()
                .try_get_matches_from(["edinet", "update", "--concurrency", "0"])
                .is_err()
        );
        assert!(
            cli_command()
                .try_get_matches_from(["edinet", "update", "--sequential", "--concurrency", "2",])
                .is_err()
        );
        assert!(
            cli_command()
                .try_get_matches_from(["edinet", "update", "--years", "2", "--today"])
                .is_err()
        );
        assert!(
            cli_command()
                .try_get_matches_from([
                    "edinet",
                    "update",
                    "--years",
                    "2",
                    "--from",
                    "2025-01-01",
                    "--to",
                    "2025-12-31",
                ])
                .is_err()
        );
    }

    #[test]
    fn update_force_is_documented() {
        let help = render_help(&["update"]);

        assert!(help.contains("-F, --force"));
        assert!(help.contains("更新済み日付を無視して対象期間をすべて再取得します"));
    }

    #[test]
    fn year_filter_is_available_on_document_commands() {
        for (path, example) in [
            (
                &["search"][..],
                "edinet search --company ストライク --year 2025",
            ),
            (&["get"][..], "edinet get --company ストライク --year 2025"),
            (
                &["download"][..],
                "edinet download pdf ./downloads --company ストライク --year 2025",
            ),
        ] {
            let help = render_help(path);
            assert!(help.contains("-y, --year <YEAR>"));
            assert!(help.contains(example));
        }

        for args in [
            &["edinet", "search", "-y", "2025"][..],
            &["edinet", "get", "-c", "ストライク", "-y", "2025"][..],
            &[
                "edinet",
                "download",
                "pdf",
                "-c",
                "ストライク",
                "-y",
                "2025",
            ][..],
        ] {
            cli_command()
                .try_get_matches_from(args)
                .unwrap_or_else(|error| panic!("year filter must parse: {error}"));
        }
    }

    #[test]
    fn year_filter_rejects_invalid_or_conflicting_values() {
        for args in [
            &["edinet", "search", "--year", "25"][..],
            &["edinet", "search", "--year", "2025", "--date", "2025-01-01"][..],
            &["edinet", "get", "--doc-id", "S100TEST", "--year", "2025"][..],
        ] {
            assert!(cli_command().try_get_matches_from(args).is_err());
        }
    }

    #[test]
    fn get_and_download_reject_removed_date_filters() {
        for args in [
            &["edinet", "get", "--date", "2025-01-01"][..],
            &["edinet", "get", "--from", "2025-01-01"][..],
            &["edinet", "get", "--to", "2025-12-31"][..],
            &["edinet", "download", "pdf", "--date", "2025-01-01"][..],
            &["edinet", "download", "pdf", "--from", "2025-01-01"][..],
            &["edinet", "download", "pdf", "--to", "2025-12-31"][..],
        ] {
            assert!(cli_command().try_get_matches_from(args).is_err());
        }
    }

    #[test]
    fn get_uses_lang_option_for_json_key_language() {
        let help = render_help(&["get"]);
        assert!(help.contains("-l, --lang <LANG>"));
        assert!(!help.contains("--keys"));

        assert!(
            cli_command()
                .try_get_matches_from(["edinet", "get", "--company", "ストライク", "--lang", "ja"])
                .is_ok()
        );
        assert!(
            cli_command()
                .try_get_matches_from(["edinet", "get", "--company", "ストライク", "--keys", "ja"])
                .is_err()
        );
    }

    #[test]
    fn command_aliases_are_accepted() {
        for args in [
            &["edinet", "init", "-h"][..],
            &["edinet", "u", "-h"][..],
            &["edinet", "s", "-h"][..],
            &["edinet", "g", "-h"][..],
            &["edinet", "d", "-h"][..],
            &["edinet", "c", "-h"][..],
            &["edinet", "st", "-h"][..],
        ] {
            cli_command()
                .try_get_matches_from(args)
                .expect_err("help flag for command alias must stop argument parsing");
        }
    }

    #[test]
    fn option_aliases_are_accepted() {
        for args in [
            &["edinet", "init", "-k", "API_KEY"][..],
            &[
                "edinet",
                "u",
                "-f",
                "2025-01-01",
                "-t",
                "2025-12-31",
                "-k",
                "API_KEY",
            ][..],
            &["edinet", "u", "-y", "2", "-p", "8", "-k", "API_KEY"][..],
            &["edinet", "u", "-y", "2", "-s", "-F", "-k", "API_KEY"][..],
            &["edinet", "u", "-d", "-k", "API_KEY"][..],
            &[
                "edinet",
                "s",
                "query",
                "-e",
                "E02144",
                "-s",
                "7203",
                "-c",
                "トヨタ",
                "-n",
                "JCN",
                "-d",
                "2025-01-01",
                "-l",
                "10",
                "-p",
                "2",
                "-o",
                "submit-date-asc",
                "-j",
            ][..],
            &["edinet", "s", "-f", "2025-01-01", "-t", "2025-12-31"][..],
            &[
                "edinet",
                "g",
                "-e",
                "E02144",
                "-c",
                "トヨタ",
                "-y",
                "2025",
                "-o",
            ][..],
            &["edinet", "g", "-i", "S100TEST", "-k", "API_KEY", "-l", "ja"][..],
            &[
                "edinet",
                "d",
                "csv",
                "-e",
                "E02144",
                "-c",
                "トヨタ",
                "-y",
                "2025",
                "-x",
            ][..],
            &["edinet", "d", "pdf", "-i", "S100TEST", "-k", "API_KEY"][..],
        ] {
            cli_command()
                .try_get_matches_from(args)
                .unwrap_or_else(|error| panic!("short options must parse: {error}"));
        }
    }

    #[test]
    fn api_key_options_use_consistent_help() {
        for path in [&["update"][..], &["get"][..], &["download"][..]] {
            assert!(
                render_help(path)
                    .contains("この実行で使う EDINET API キー（登録済みキーより優先）")
            );
        }
    }

    #[test]
    fn short_and_long_help_flags_have_identical_output() {
        for command in [
            &[][..],
            &["setup"][..],
            &["update"][..],
            &["search"][..],
            &["get"][..],
            &["download"][..],
            &["clear"][..],
            &["status"][..],
        ] {
            let mut short_args = vec!["edinet"];
            short_args.extend_from_slice(command);
            short_args.push("-h");

            let mut long_args = vec!["edinet"];
            long_args.extend_from_slice(command);
            long_args.push("--help");

            assert_eq!(
                help_output(&short_args),
                help_output(&long_args),
                "help output differs for `{}`",
                command.join(" ")
            );
        }
    }
}
