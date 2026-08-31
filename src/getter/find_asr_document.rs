// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use sea_orm::DatabaseConnection;

use crate::store::asr_document_metadata::AsrDocumentMetadata;

/// 条件に一致する最新の有価証券報告書 metadata を返す。
///
/// `None` の場合、条件に一致する CSV 取得可能な対象が保存済み一覧に存在しない。
pub async fn find_asr_document_metadata(
    db: &DatabaseConnection,
    edinet_code: Option<&str>,
    filer_name: Option<&str>,
    submitted_year: Option<u16>,
) -> anyhow::Result<Option<AsrDocumentMetadata>> {
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
