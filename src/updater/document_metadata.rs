// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// EDINET 日次書類一覧 API の `results` 1 件を表す型。
pub struct DocumentMetadata {
    #[serde(rename = "seqNumber")]
    pub seq_number: i64,

    #[serde(rename = "docID")]
    pub doc_id: String,

    #[serde(rename = "edinetCode")]
    pub edinet_code: Option<String>,

    #[serde(rename = "secCode")]
    pub sec_code: Option<String>,

    #[serde(rename = "JCN")]
    pub jcn: Option<String>,

    #[serde(rename = "filerName")]
    pub filer_name: Option<String>,

    #[serde(rename = "ordinanceCode")]
    pub ordinance_code: Option<String>,

    #[serde(rename = "formCode")]
    pub form_code: Option<String>,

    #[serde(rename = "docTypeCode")]
    pub doc_type_code: Option<String>,

    #[serde(rename = "periodStart")]
    pub period_start: Option<String>,

    #[serde(rename = "periodEnd")]
    pub period_end: Option<String>,

    #[serde(rename = "submitDateTime")]
    pub submit_date_time: Option<String>,

    #[serde(rename = "docDescription")]
    pub doc_description: Option<String>,

    #[serde(rename = "withdrawalStatus")]
    pub withdrawal_status: String,

    #[serde(rename = "docInfoEditStatus")]
    pub doc_info_edit_status: String,

    #[serde(rename = "disclosureStatus")]
    pub disclosure_status: String,

    #[serde(rename = "xbrlFlag")]
    pub xbrl_flag: String,

    #[serde(rename = "pdfFlag")]
    pub pdf_flag: String,

    #[serde(rename = "attachDocFlag")]
    pub attach_doc_flag: String,

    #[serde(rename = "englishDocFlag")]
    pub english_doc_flag: String,

    #[serde(rename = "csvFlag")]
    pub csv_flag: String,

    #[serde(rename = "legalStatus")]
    pub legal_status: String,
}
