// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use crate::store::entities::document_metadata;

/// インデックス DB (asrs.db) の `document_metadatas` テーブルの各レコードの写像に値する、有報メタデータのアプリ内部型。
///
/// 保存済み DB レコードをアプリ内部で扱うための型。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AsrDocumentMetadata {
    /// DB レコードの主キー。
    pub id: i64,
    /// EDINET の日次書類一覧上の提出日。
    pub file_date: String,
    /// 同一提出日の一覧内連番。
    pub seq_number: i64,
    /// EDINET 書類 ID。
    pub doc_id: String,
    /// 提出者の EDINET コード。
    pub edinet_code: Option<String>,
    /// 提出者の証券コード。
    pub sec_code: Option<String>,
    /// 提出者の法人番号。
    pub jcn: Option<String>,
    /// 提出者名。
    pub filer_name: Option<String>,
    /// 府令コード。
    pub ordinance_code: Option<String>,
    /// 様式コード。
    pub form_code: Option<String>,
    /// 書類種別コード。
    pub doc_type_code: Option<String>,
    /// 対象事業年度の開始日。
    pub period_start: Option<String>,
    /// 対象事業年度の終了日。
    pub period_end: Option<String>,
    /// EDINET 上の提出日時。
    pub submit_date_time: Option<String>,
    /// 書類名や対象期間を含む書類概要。
    pub doc_description: Option<String>,
    /// 取下げ状況。
    pub withdrawal_status: String,
    /// 書類情報修正状況。
    pub doc_info_edit_status: String,
    /// 開示状況。
    pub disclosure_status: String,
    /// XBRL ファイルの有無を表すフラグ。
    pub xbrl_flag: String,
    /// PDF ファイルの有無を表すフラグ。
    pub pdf_flag: String,
    /// 添付書類の有無を表すフラグ。
    pub attach_doc_flag: String,
    /// 英文書類の有無を表すフラグ。
    pub english_doc_flag: String,
    /// CSV ファイルの有無を表すフラグ。
    pub csv_flag: String,
    /// 縦覧状況。
    pub legal_status: String,
}

impl From<document_metadata::Model> for AsrDocumentMetadata {
    fn from(row: document_metadata::Model) -> Self {
        Self {
            id: row.id,
            file_date: row.file_date,
            seq_number: row.seq_number,
            doc_id: row.doc_id,
            edinet_code: row.edinet_code,
            sec_code: row.sec_code,
            jcn: row.jcn,
            filer_name: row.filer_name,
            ordinance_code: row.ordinance_code,
            form_code: row.form_code,
            doc_type_code: row.doc_type_code,
            period_start: row.period_start,
            period_end: row.period_end,
            submit_date_time: row.submit_date_time,
            doc_description: row.doc_description,
            withdrawal_status: row.withdrawal_status,
            doc_info_edit_status: row.doc_info_edit_status,
            disclosure_status: row.disclosure_status,
            xbrl_flag: row.xbrl_flag,
            pdf_flag: row.pdf_flag,
            attach_doc_flag: row.attach_doc_flag,
            english_doc_flag: row.english_doc_flag,
            csv_flag: row.csv_flag,
            legal_status: row.legal_status,
        }
    }
}
