// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use tabled::Tabled;

use crate::store::asr_document_metadata::AsrDocumentMetadata;

/// `search` コマンドの表形式の標準出力。
#[derive(Debug)]
pub struct SearchCommandOutputTable {
    rows: Vec<SearchCommandOutputTableRow>,
}

impl SearchCommandOutputTable {
    pub fn new(metadatas: &[AsrDocumentMetadata]) -> Self {
        Self {
            rows: metadatas
                .iter()
                .map(SearchCommandOutputTableRow::from)
                .collect(),
        }
    }
}

impl std::fmt::Display for SearchCommandOutputTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = tabled::Table::new(&self.rows);
        table.with(tabled::settings::Style::empty());

        write!(f, "{}", table)
    }
}

#[derive(Debug, Tabled)]
struct SearchCommandOutputTableRow {
    #[tabled(rename = "書類ID")]
    doc_id: String,
    #[tabled(rename = "EDINETコード")]
    edinet_code: String,
    #[tabled(rename = "証券コード")]
    sec_code: String,
    #[tabled(rename = "法人番号")]
    jcn: String,
    #[tabled(rename = "提出者名")]
    filer_name: String,
    #[tabled(rename = "期間開始日")]
    period_start: String,
    #[tabled(rename = "期間終了日")]
    period_end: String,
    #[tabled(rename = "提出日時")]
    submit_date_time: String,
    #[tabled(rename = "書類概要")]
    doc_description: String,
}

impl From<&AsrDocumentMetadata> for SearchCommandOutputTableRow {
    fn from(value: &AsrDocumentMetadata) -> Self {
        SearchCommandOutputTableRow {
            doc_id: value.doc_id.clone(),
            edinet_code: value.edinet_code.clone().unwrap_or_default(),
            sec_code: value.sec_code.clone().unwrap_or_default(),
            jcn: value.jcn.clone().unwrap_or_default(),
            filer_name: value.filer_name.clone().unwrap_or_default(),
            period_start: value.period_start.clone().unwrap_or_default(),
            period_end: value.period_end.clone().unwrap_or_default(),
            submit_date_time: value.submit_date_time.clone().unwrap_or_default(),
            doc_description: value.doc_description.clone().unwrap_or_default(),
        }
    }
}
