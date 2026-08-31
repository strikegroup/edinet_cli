use clap::{ArgGroup, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputLanguage {
    En,
    Ja,
}

#[derive(clap::Args, Debug)]
#[command(group(
    ArgGroup::new("filter")
        .args(["edinet_code", "company", "submitted_year"])
        .multiple(true)
))]
pub struct GetArgs {
    #[arg(long, short = 'e', help = "EDINET コードで対象書類を絞り込みます")]
    pub edinet_code: Option<String>,
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
}

pub async fn run(args: GetArgs) -> anyhow::Result<()> {
    let api_key = if args.offline {
        None
    } else {
        Some(crate::app_config::resolve_api_key(args.key.as_deref())?)
    };
    if args.edinet_code.is_none()
        && args.company.is_none()
        && args.submitted_year.is_none()
        && args.doc_id.is_none()
    {
        return Err(anyhow::anyhow!(
            "At least one query parameter must be specified"
        ));
    }

    if args.doc_id.is_some()
        && (args.edinet_code.is_some() || args.company.is_some() || args.submitted_year.is_some())
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
        print_get_output(None, &report, args.lang)?;
        return Ok(());
    }

    let db = crate::store::open_db::open_db_connection().await?;
    let metadata = crate::getter::find_asr_document::find_asr_document_metadata(
        &db,
        args.edinet_code.as_deref(),
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
    print_get_output(Some(&metadata), &report, args.lang)?;

    Ok(())
}

fn print_get_output(
    metadata: Option<&crate::store::asr_document_metadata::AsrDocumentMetadata>,
    report: &crate::getter::asr_report::AsrReport,
    language: OutputLanguage,
) -> anyhow::Result<()> {
    if matches!(language, OutputLanguage::Ja) {
        let output =
            crate::getter::output::get_command_output_ja::GetCommandOutputJa::new(metadata, report);
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let output = crate::getter::output::get_command_output::GetCommandOutput::new(metadata, report);
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
