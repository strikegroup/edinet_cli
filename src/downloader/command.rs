use clap::ArgGroup;

use crate::downloader::download_document::DocType;

#[derive(clap::Args, Debug)]
#[command(group(
    ArgGroup::new("filter")
        .args(["edinet_code", "company", "submitted_year"])
        .multiple(true)
))]
pub struct DownloadArgs {
    #[arg(display_order = 1, help = "ダウンロードする書類形式")]
    pub doc_type: DocType,
    #[arg(
        display_order = 2,
        help = "出力先のファイルまたはディレクトリ",
        value_name = "PATH"
    )]
    pub dest: Option<std::path::PathBuf>,
    #[arg(
        long,
        short = 'i',
        value_name = "DOC_ID",
        value_parser = crate::document_id::parse,
        help = "書類 ID を直接指定します（ほかの検索条件と併用不可）"
    )]
    pub doc_id: Option<String>,
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
        short = 'x',
        help = "ダウンロードした XBRL または CSV の ZIP を展開します"
    )]
    pub extract: bool,
    #[arg(
        long,
        short = 'k',
        value_name = "API_KEY",
        help = "この実行で使う EDINET API キー（登録済みキーより優先）"
    )]
    pub key: Option<String>,
}

pub async fn run(args: DownloadArgs) -> anyhow::Result<()> {
    if args.doc_id.is_none()
        && args.edinet_code.is_none()
        && args.company.is_none()
        && args.submitted_year.is_none()
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

    let doc_id = match args.doc_id {
        Some(doc_id) => doc_id,
        None => {
            let db = crate::store::open_db::open_db_connection().await?;
            let metadata = find_asr_document_metadata(
                &db,
                args.edinet_code.as_deref(),
                args.company.as_deref(),
                args.submitted_year.map(|year| year.get()),
            )
            .await?
            .ok_or_else(|| anyhow::anyhow!("ASR document not found for the given query"))?;
            metadata.doc_id
        }
    };
    let normalized_dest = args.dest.unwrap_or(std::env::current_dir()?);
    let api_key = crate::app_config::resolve_api_key(args.key.as_deref())?;
    crate::downloader::download_document::download_document(
        &args.doc_type,
        &doc_id,
        &api_key,
        &normalized_dest,
        args.extract,
    )
    .await?;

    Ok(())
}

async fn find_asr_document_metadata(
    db: &sea_orm::DatabaseConnection,
    edinet_code: Option<&str>,
    filer_name: Option<&str>,
    submitted_year: Option<u16>,
) -> anyhow::Result<Option<crate::store::asr_document_metadata::AsrDocumentMetadata>> {
    let condition = crate::searcher::search_asr_documents::SearchCondition {
        query: None,
        query_sec_code: None,
        edinet_code: edinet_code.map(str::to_owned),
        sec_code: None,
        jcn: None,
        filer_name: filer_name.map(str::to_owned),
        submitted_date: None,
        submitted_from: None,
        submitted_to: None,
        submitted_year,
        limit: Some(1),
        offset: 0,
        sort: crate::searcher::search_asr_documents::SearchSort::SubmitDateDesc,
    };
    let mut metadatas =
        crate::searcher::search_asr_documents::search_asr_document_metadatas(db, &condition)
            .await?;

    Ok(metadatas.pop())
}
