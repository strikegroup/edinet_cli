// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use clap::{ArgGroup, ValueEnum};
use jaq_all::jaq_core::unwrap_valr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputLanguage {
    En,
    Ja,
}

#[derive(clap::Args, Debug)]
#[command(group(
    ArgGroup::new("filter")
        .args(["edinet_code", "sec_code", "company", "submitted_year"])
        .multiple(true)
))]
pub struct GetArgs {
    #[arg(long, short = 'e', help = "EDINET コードで対象書類を絞り込みます")]
    pub edinet_code: Option<String>,
    #[arg(long, short = 's', help = "証券コードで対象書類を絞り込みます")]
    pub sec_code: Option<String>,
    #[arg(
        long,
        short = 'c',
        value_name = "COMPANY",
        help = "会社名または提出者名で対象書類を絞り込みます"
    )]
    pub company: Option<String>,
    #[arg(
        long = "year",
        short = 'y',
        value_name = "YEAR",
        help = "提出年で対象書類を絞り込みます（4 桁の西暦）"
    )]
    pub submitted_year: Option<crate::submission_year::SubmissionYear>,
    #[arg(
        long,
        short = 'i',
        value_name = "DOC_ID",
        value_parser = crate::document_id::parse,
        conflicts_with = "filter",
        help = "書類 ID を直接指定します（ほかの検索条件と併用不可）"
    )]
    pub doc_id: Option<String>,
    #[arg(
        long,
        short = 'k',
        value_name = "API_KEY",
        help = "この実行で使う EDINET API キー（登録済みキーより優先）"
    )]
    pub key: Option<String>,
    #[arg(long, short = 'o', help = "保存済み CSV キャッシュだけを使用します")]
    pub offline: bool,
    #[arg(long, short = 'l', value_enum, default_value_t = OutputLanguage::En, help = "JSON のキー言語を指定します（en または ja）")]
    pub lang: OutputLanguage,
    #[arg(
        long,
        short = 'f',
        value_name = "FILTER",
        help = "jq 互換フィルタを適用して JSON の出力項目を選択します"
    )]
    pub format: Option<String>,
}

pub async fn run(args: GetArgs) -> anyhow::Result<()> {
    let api_key = if args.offline {
        None
    } else {
        Some(crate::app_config::resolve_api_key(args.key.as_deref())?)
    };
    if args.edinet_code.is_none()
        && args.sec_code.is_none()
        && args.company.is_none()
        && args.submitted_year.is_none()
        && args.doc_id.is_none()
    {
        return Err(anyhow::anyhow!(
            "At least one query parameter must be specified"
        ));
    }

    if args.doc_id.is_some()
        && (args.edinet_code.is_some()
            || args.sec_code.is_some()
            || args.company.is_some()
            || args.submitted_year.is_some())
    {
        return Err(anyhow::anyhow!(
            "--doc-id cannot be combined with other query parameters"
        ));
    }

    if let Some(doc_id) = args.doc_id {
        let report = crate::getter::load_asr_report::load_asr_report_by_doc_id(
            &doc_id,
            api_key.as_deref(),
            args.offline,
        )
        .await?;
        print_get_output(None, &report, args.lang, args.format.as_deref())?;
        return Ok(());
    }

    let db = crate::store::open_db::open_db_connection().await?;
    let metadata = crate::getter::find_asr_document::find_asr_document_metadata(
        &db,
        args.edinet_code.as_deref(),
        args.sec_code.as_deref(),
        args.company.as_deref(),
        args.submitted_year.map(|year| year.get()),
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("ASR document not found for the given query"))?;

    let report = crate::getter::load_asr_report::load_asr_report_by_metadata(
        &metadata,
        api_key.as_deref(),
        args.offline,
    )
    .await?;
    print_get_output(Some(&metadata), &report, args.lang, args.format.as_deref())?;

    Ok(())
}

fn print_get_output(
    metadata: Option<&crate::store::asr_document_metadata::AsrDocumentMetadata>,
    report: &crate::getter::asr_report::AsrReport,
    language: OutputLanguage,
    jq_filter: Option<&str>,
) -> anyhow::Result<()> {
    if matches!(language, OutputLanguage::Ja) {
        let output =
            crate::getter::output::get_command_output_ja::GetCommandOutputJa::new(metadata, report);
        print!("{}", render_output(&output, jq_filter)?);
        return Ok(());
    }

    let output = crate::getter::output::get_command_output::GetCommandOutput::new(metadata, report);
    print!("{}", render_output(&output, jq_filter)?);
    Ok(())
}

fn render_output(
    output: &impl serde::Serialize,
    jq_filter: Option<&str>,
) -> anyhow::Result<String> {
    let json = serde_json::to_string_pretty(output)?;
    let Some(jq_filter) = jq_filter else {
        return Ok(format!("{json}\n"));
    };

    apply_jq_filter(&json, jq_filter)
}

fn apply_jq_filter(json: &str, jq_filter: &str) -> anyhow::Result<String> {
    let filter = jaq_all::compile_with(jq_filter, jaq_all::defs(), jaq_all::data::base_funs(), &[])
        .map_err(|reports| {
            let details = reports
                .iter()
                .map(|report| jaq_all::load::FileReportsDisp::new(report).to_string())
                .collect::<Vec<_>>()
                .join("");
            anyhow::anyhow!("invalid jq filter:\n{details}")
        })?;
    let input = jaq_all::json::read::parse_single(json.as_bytes())
        .map_err(|error| anyhow::anyhow!("failed to prepare JSON for jq filter: {error}"))?;

    let mut runner = jaq_all::data::Runner::default();
    runner.writer.pp.indent = Some("  ".to_owned());
    runner.writer.pp.sep_space = true;

    let mut rendered = Vec::new();
    jaq_all::data::run(
        &runner,
        &filter,
        Default::default(),
        std::iter::once(Ok::<_, String>(input)),
        |error| anyhow::anyhow!(error),
        |value| {
            let value =
                unwrap_valr(value).map_err(|error| anyhow::anyhow!("jq filter failed: {error}"))?;
            jaq_all::json::write::write(&mut rendered, &runner.writer.pp, 0, &value)?;
            rendered.push(b'\n');
            Ok(())
        },
    )?;

    String::from_utf8(rendered)
        .map_err(|error| anyhow::anyhow!("jq filter produced invalid UTF-8: {error}"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::render_output;

    #[test]
    fn formatでネストしたフィールドを抽出できる() {
        let output = json!({"foo": {"bar": {"buz": 42}}, "other": true});

        let rendered = render_output(&output, Some(".foo.bar.buz")).unwrap();

        assert_eq!(rendered, "42\n");
    }

    #[test]
    fn formatは複数の結果をjqと同様に一件ずつ出力する() {
        let output = json!({"items": [{"name": "A"}, {"name": "B"}]});

        let rendered = render_output(&output, Some(".items[].name")).unwrap();

        assert_eq!(rendered, "\"A\"\n\"B\"\n");
    }

    #[test]
    fn formatを省略すると従来どおりjsonを整形して出力する() {
        let output = json!({"foo": {"bar": 1}});

        let rendered = render_output(&output, None).unwrap();

        assert_eq!(rendered, "{\n  \"foo\": {\n    \"bar\": 1\n  }\n}\n");
    }

    #[test]
    fn 不正なformatはエラーにする() {
        let error = render_output(&json!({"foo": 1}), Some(".foo |")).unwrap_err();

        assert!(error.to_string().contains("invalid jq filter"));
    }
}
