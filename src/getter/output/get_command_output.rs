use crate::getter::asr_report::AsrReport;
use crate::store::asr_document_metadata::AsrDocumentMetadata;

/// `get` コマンドの標準出力。
#[derive(Debug, serde::Serialize)]
pub struct GetCommandOutput<'a> {
    pub metadata: Option<GetCommandMetadataOutput<'a>>,
    pub report: &'a AsrReport,
}

impl<'a> GetCommandOutput<'a> {
    pub fn new(metadata: Option<&'a AsrDocumentMetadata>, report: &'a AsrReport) -> Self {
        Self {
            metadata: metadata.map(GetCommandMetadataOutput::from),
            report,
        }
    }
}

/// `get` コマンドの JSON 出力に含める書類 metadata。
#[derive(Debug, serde::Serialize)]
pub struct GetCommandMetadataOutput<'a> {
    pub file_date: &'a str,
    pub doc_id: &'a str,
    pub edinet_code: Option<&'a str>,
    pub sec_code: Option<&'a str>,
    pub filer_name: Option<&'a str>,
    pub period_start: Option<&'a str>,
    pub period_end: Option<&'a str>,
    pub submit_date_time: Option<&'a str>,
    pub doc_description: Option<&'a str>,
}

impl<'a> From<&'a AsrDocumentMetadata> for GetCommandMetadataOutput<'a> {
    fn from(value: &'a AsrDocumentMetadata) -> Self {
        Self {
            file_date: &value.file_date,
            doc_id: &value.doc_id,
            edinet_code: value.edinet_code.as_deref(),
            sec_code: value.sec_code.as_deref(),
            filer_name: value.filer_name.as_deref(),
            period_start: value.period_start.as_deref(),
            period_end: value.period_end.as_deref(),
            submit_date_time: value.submit_date_time.as_deref(),
            doc_description: value.doc_description.as_deref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::getter::asr_report::AsrReport;

    #[test]
    fn serializes_default_english_keys() {
        let report = AsrReport::default();
        let value = serde_json::to_value(GetCommandOutput::new(None, &report))
            .expect("default output should serialize");

        assert!(value.get("metadata").is_some());
        assert!(value.get("report").is_some());
    }
}
