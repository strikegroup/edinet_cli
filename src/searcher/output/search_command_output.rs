// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use crate::store::asr_document_metadata::AsrDocumentMetadata;

/// `search` コマンドの標準出力。
#[derive(Debug, serde::Serialize)]
pub struct SearchCommandOutput<'a> {
    pub metadatas: Vec<SearchCommandMetadataOutput<'a>>,
}

impl<'a> SearchCommandOutput<'a> {
    pub fn new(metadatas: &'a [AsrDocumentMetadata]) -> Self {
        Self {
            metadatas: metadatas
                .iter()
                .map(SearchCommandMetadataOutput::from)
                .collect(),
        }
    }
}

/// `search` コマンドの JSON 出力に含める書類 metadata。
#[derive(Debug, serde::Serialize)]
pub struct SearchCommandMetadataOutput<'a> {
    pub doc_id: &'a str,
    pub edinet_code: Option<&'a str>,
    pub sec_code: Option<&'a str>,
    pub jcn: Option<&'a str>,
    pub filer_name: Option<&'a str>,
    pub period_start: Option<&'a str>,
    pub period_end: Option<&'a str>,
    pub submit_date_time: Option<&'a str>,
    pub doc_description: Option<&'a str>,
}

impl<'a> From<&'a AsrDocumentMetadata> for SearchCommandMetadataOutput<'a> {
    fn from(value: &'a AsrDocumentMetadata) -> Self {
        Self {
            doc_id: &value.doc_id,
            edinet_code: value.edinet_code.as_deref(),
            sec_code: value.sec_code.as_deref(),
            jcn: value.jcn.as_deref(),
            filer_name: value.filer_name.as_deref(),
            period_start: value.period_start.as_deref(),
            period_end: value.period_end.as_deref(),
            submit_date_time: value.submit_date_time.as_deref(),
            doc_description: value.doc_description.as_deref(),
        }
    }
}
