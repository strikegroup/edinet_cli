// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};

use crate::store::entities::{document_metadata, updated_document_metadata};

const SQLITE_MAX_BIND_VARIABLES: usize = 999;
const DOCUMENT_METADATA_INSERT_COLUMNS: usize = 23;
const DOCUMENT_METADATA_INSERT_CHUNK_SIZE: usize =
    SQLITE_MAX_BIND_VARIABLES / DOCUMENT_METADATA_INSERT_COLUMNS;

/// 取得した 1 日分の書類メタデータを DB に保存する。
///
/// 同一 `file_date` の既存メタデータは削除してから再投入する。
pub async fn save_document_metadatas(
    db: &DatabaseConnection,
    file_date: &chrono::NaiveDate,
    fetched: &crate::updater::fetch_document_metadatas::FetchedDocumentMetadatas,
) -> anyhow::Result<()> {
    let updated_at = chrono::Utc::now().to_rfc3339();
    let file_date_string = file_date.to_string();
    let result_count =
        i64::try_from(fetched.results.len()).context("failed to convert result count to i64")?;

    let txn = db
        .begin()
        .await
        .with_context(|| format!("failed to start transaction for {}", file_date))?;

    document_metadata::Entity::delete_many()
        .filter(document_metadata::Column::FileDate.eq(file_date_string.clone()))
        .exec(&txn)
        .await
        .with_context(|| {
            format!(
                "failed to delete existing document metadata rows for {}",
                file_date
            )
        })?;

    for items in fetched.results.chunks(DOCUMENT_METADATA_INSERT_CHUNK_SIZE) {
        let rows = items
            .iter()
            .map(|item| document_metadata_active_model(&file_date_string, item));

        document_metadata::Entity::insert_many(rows)
            .exec(&txn)
            .await
            .with_context(|| {
                format!("failed to insert document metadata rows for {}", file_date)
            })?;
    }

    updated_document_metadata::Entity::insert(updated_document_metadata::ActiveModel {
        file_date: Set(file_date_string),
        updated_at: Set(updated_at),
        result_count: Set(result_count),
    })
    .on_conflict(
        OnConflict::column(updated_document_metadata::Column::FileDate)
            .update_columns([
                updated_document_metadata::Column::UpdatedAt,
                updated_document_metadata::Column::ResultCount,
            ])
            .to_owned(),
    )
    .exec(&txn)
    .await
    .with_context(|| {
        format!(
            "failed to upsert updated document metadata status for {}",
            file_date
        )
    })?;

    txn.commit()
        .await
        .with_context(|| format!("failed to commit transaction for {}", file_date))?;

    Ok(())
}

fn document_metadata_active_model(
    file_date: &str,
    item: &crate::updater::document_metadata::DocumentMetadata,
) -> document_metadata::ActiveModel {
    document_metadata::ActiveModel {
        file_date: Set(file_date.to_owned()),
        seq_number: Set(item.seq_number),
        doc_id: Set(item.doc_id.clone()),
        edinet_code: Set(item.edinet_code.clone()),
        sec_code: Set(item.sec_code.clone()),
        jcn: Set(item.jcn.clone()),
        filer_name: Set(item.filer_name.clone()),
        ordinance_code: Set(item.ordinance_code.clone()),
        form_code: Set(item.form_code.clone()),
        doc_type_code: Set(item.doc_type_code.clone()),
        period_start: Set(item.period_start.clone()),
        period_end: Set(item.period_end.clone()),
        submit_date_time: Set(item.submit_date_time.clone()),
        doc_description: Set(item.doc_description.clone()),
        withdrawal_status: Set(item.withdrawal_status.clone()),
        doc_info_edit_status: Set(item.doc_info_edit_status.clone()),
        disclosure_status: Set(item.disclosure_status.clone()),
        xbrl_flag: Set(item.xbrl_flag.clone()),
        pdf_flag: Set(item.pdf_flag.clone()),
        attach_doc_flag: Set(item.attach_doc_flag.clone()),
        english_doc_flag: Set(item.english_doc_flag.clone()),
        csv_flag: Set(item.csv_flag.clone()),
        legal_status: Set(item.legal_status.clone()),
        ..Default::default()
    }
}
