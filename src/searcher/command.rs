// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use clap::ValueEnum;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum SearchSortArg {
    SubmitDateDesc,
    SubmitDateAsc,
}

impl From<SearchSortArg> for crate::searcher::search_asr_documents::SearchSort {
    fn from(value: SearchSortArg) -> Self {
        match value {
            SearchSortArg::SubmitDateDesc => {
                crate::searcher::search_asr_documents::SearchSort::SubmitDateDesc
            }
            SearchSortArg::SubmitDateAsc => {
                crate::searcher::search_asr_documents::SearchSort::SubmitDateAsc
            }
        }
    }
}

#[derive(clap::Args, Debug)]
pub struct SearchArgs {
    #[arg(
        value_name = "QUERY",
        help = "提出者名、EDINET コード、証券コード、法人番号をまとめて検索します"
    )]
    pub query: Option<String>,
    #[arg(long, short = 'e', help = "EDINET コードで絞り込みます")]
    pub edinet_code: Option<String>,
    #[arg(
        long,
        short = 's',
        help = "証券コードで絞り込みます（4 桁または 5 桁）"
    )]
    pub sec_code: Option<String>,
    #[arg(
        long = "company",
        short = 'c',
        value_name = "COMPANY",
        help = "会社名または提出者名で絞り込みます"
    )]
    pub filer_name: Option<String>,
    #[arg(long, short = 'n', help = "法人番号で絞り込みます")]
    pub jcn: Option<String>,
    #[arg(
        long = "date",
        short = 'd',
        value_name = "DATE",
        help = "提出日を指定します（YYYY-MM-DD、`--from`・`--to` と併用不可）"
    )]
    pub submitted_date: Option<String>,
    #[arg(
        long = "from",
        short = 'f',
        value_name = "DATE",
        help = "提出日の開始日を指定します（YYYY-MM-DD）"
    )]
    pub submitted_from: Option<String>,
    #[arg(
        long = "to",
        short = 't',
        value_name = "DATE",
        help = "提出日の終了日を指定します（YYYY-MM-DD）"
    )]
    pub submitted_to: Option<String>,
    #[arg(
        long = "year",
        short = 'y',
        value_name = "YEAR",
        conflicts_with_all = ["submitted_date", "submitted_from", "submitted_to"],
        help = "提出年で絞り込みます（4 桁の西暦）"
    )]
    pub submitted_year: Option<crate::submission_year::SubmissionYear>,
    #[arg(
        long,
        short = 'l',
        default_value_t = 0,
        help = "1 ページあたりの表示件数を指定します（0 は全件表示）"
    )]
    pub limit: u32,
    #[arg(
        long,
        short = 'p',
        default_value_t = 1,
        help = "表示するページ番号を指定します"
    )]
    pub page: u32,
    #[arg(long, short = 'o', value_enum, default_value_t = SearchSortArg::SubmitDateDesc, help = "検索結果の並び順を指定します")]
    pub sort: SearchSortArg,
    #[arg(long, short = 'j', help = "検索結果を JSON 形式で出力します")]
    pub json: bool,
}

pub async fn run(args: SearchArgs) -> anyhow::Result<()> {
    let (condition, output_json) = build_search_condition(args)?;

    let db = crate::store::open_db::open_db_connection().await?;
    let metadatas =
        crate::searcher::search_asr_documents::search_asr_document_metadatas(&db, &condition)
            .await?;

    match output_json {
        true => {
            let output = crate::searcher::output::search_command_output::SearchCommandOutput::new(
                &metadatas,
            );

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        false => {
            let output =
                crate::searcher::output::search_command_output_table::SearchCommandOutputTable::new(
                    &metadatas,
                );

            println!("{}", output);
        }
    }

    Ok(())
}

fn build_search_condition(
    args: SearchArgs,
) -> anyhow::Result<(crate::searcher::search_asr_documents::SearchCondition, bool)> {
    if args.submitted_year.is_some()
        && (args.submitted_date.is_some()
            || args.submitted_from.is_some()
            || args.submitted_to.is_some())
    {
        return Err(anyhow::anyhow!(
            "--year cannot be combined with --date, --from, or --to"
        ));
    }
    let submitted_date = parse_optional_date(args.submitted_date)?;
    let submitted_from = parse_optional_date(args.submitted_from)?;
    let submitted_to = parse_optional_date(args.submitted_to)?;

    if args.limit > 100 {
        return Err(anyhow::anyhow!(
            "--limit must be between 0 and 100; 0 means no limit"
        ));
    }
    if args.page == 0 {
        return Err(anyhow::anyhow!("--page must be 1 or greater"));
    }
    if submitted_date.is_some() && (submitted_from.is_some() || submitted_to.is_some()) {
        return Err(anyhow::anyhow!(
            "--date cannot be combined with --from or --to"
        ));
    }
    if let (Some(from), Some(to)) = (&submitted_from, &submitted_to) {
        if from > to {
            return Err(anyhow::anyhow!(
                "--from must be earlier than or equal to --to"
            ));
        }
    }

    let limit = if args.limit == 0 {
        None
    } else {
        Some(u64::from(args.limit))
    };
    let offset = limit.map_or(0, |limit| u64::from(args.page - 1) * limit);
    let query = non_empty(args.query);

    Ok((
        crate::searcher::search_asr_documents::SearchCondition {
            query: query.clone(),
            query_sec_code: query.map(normalize_sec_code),
            edinet_code: non_empty(args.edinet_code),
            sec_code: non_empty(args.sec_code).map(normalize_sec_code),
            jcn: non_empty(args.jcn),
            filer_name: non_empty(args.filer_name),
            submitted_date,
            submitted_from,
            submitted_to,
            submitted_year: args.submitted_year.map(|year| year.get()),
            limit,
            offset,
            sort: args.sort.into(),
        },
        args.json,
    ))
}

fn parse_date(date: &str) -> anyhow::Result<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .with_context(|| format!("failed to parse date: {}", date))
}

fn parse_optional_date(date: Option<String>) -> anyhow::Result<Option<String>> {
    let Some(date) = date else {
        return Ok(None);
    };
    parse_date(&date)?;
    Ok(Some(date))
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_owned())
        }
    })
}

fn normalize_sec_code(value: String) -> String {
    if value.len() == 4 && value.bytes().all(|byte| byte.is_ascii_digit()) {
        format!("{value}0")
    } else {
        value
    }
}
