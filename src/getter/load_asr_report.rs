// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;

use super::asr_report::AsrReport;
use crate::store::asr_document_metadata::AsrDocumentMetadata;

/// `AsrDocumentMetadata` の doc_id を使って有価証券報告書を読み込む。
pub async fn load_asr_report_by_metadata(
    metadata: &AsrDocumentMetadata,
    api_key: Option<&str>,
    offline: bool,
) -> anyhow::Result<AsrReport> {
    let doc_id = metadata.doc_id.clone();
    load_asr_report_by_doc_id(&doc_id, api_key, offline).await
}

/// `doc_id` のみを使って有価証券報告書を読み込む。
pub async fn load_asr_report_by_doc_id(
    doc_id: &str,
    api_key: Option<&str>,
    offline: bool,
) -> anyhow::Result<AsrReport> {
    crate::document_id::validate(doc_id)?;
    let cache_root = cache_root_dir()?;
    let csv_path = if offline {
        find_cached_asr_csv(doc_id, &cache_root)?
    } else {
        let api_key = api_key.ok_or_else(|| anyhow::anyhow!("EDINET API key is required"))?;
        find_or_download_asr_csv(doc_id, &cache_root, api_key).await?
    };
    let mut reader = super::load_local_csv::load_csv_from_path(&csv_path)?;

    super::extract_asr_report::extract_asr_report_from_csv(&mut reader)
}

fn cache_root_dir() -> anyhow::Result<std::path::PathBuf> {
    if let Ok(path) = std::env::var("ASRS_CSV_CACHE_DIR") {
        return Ok(std::path::PathBuf::from(path));
    }

    crate::app_paths::default_csv_cache_dir()
}

fn find_cached_asr_csv(
    doc_id: &str,
    cache_root: &std::path::Path,
) -> anyhow::Result<std::path::PathBuf> {
    let extract_dir = cache_root.join(doc_id);
    find_asr_csv_path(&extract_dir)
        .map_err(|_| anyhow::anyhow!("ASR CSV cache miss for {} at {:?}", doc_id, extract_dir))
}

async fn find_or_download_asr_csv(
    doc_id: &str,
    cache_root: &std::path::Path,
    api_key: &str,
) -> anyhow::Result<std::path::PathBuf> {
    let extract_dir = cache_root.join(doc_id);
    if let Ok(path) = find_asr_csv_path(&extract_dir) {
        return Ok(path);
    }

    super::download_document::download_and_extract_xbrl_csv_archive(
        doc_id,
        api_key,
        cache_root,
        &extract_dir,
    )
    .await?;
    find_asr_csv_path(&extract_dir)
}

fn find_asr_csv_path(extract_dir: &std::path::Path) -> anyhow::Result<std::path::PathBuf> {
    let csv_dir = extract_dir.join("XBRL_TO_CSV");
    let entries = std::fs::read_dir(&csv_dir)
        .with_context(|| format!("failed to read XBRL_TO_CSV directory at {:?}", csv_dir))?;

    for entry in entries {
        let entry = entry.context("failed to read CSV directory entry")?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if file_name.ends_with(".csv") && file_name.to_ascii_lowercase().contains("asr") {
            return Ok(path);
        }
    }

    Err(anyhow::anyhow!("ASR CSV file not found"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_asr_csv_file_name() -> anyhow::Result<()> {
        let temp_dir = std::env::temp_dir().join(format!(
            "asrs_getter_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let csv_dir = temp_dir.join("XBRL_TO_CSV");
        std::fs::create_dir_all(&csv_dir)?;
        let target = csv_dir.join("jpcrp030000-asr-001_E00000-000_2026-03-31_01_2026-04-01.csv");
        std::fs::write(&target, [])?;

        let found = find_asr_csv_path(&temp_dir)?;
        assert_eq!(found, target);

        std::fs::remove_dir_all(temp_dir)?;
        Ok(())
    }

    #[test]
    fn offline_mode_fails_on_cache_miss() -> anyhow::Result<()> {
        let temp_dir = std::env::temp_dir().join(format!(
            "asrs_getter_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));

        let result = find_cached_asr_csv("S100MISSING", &temp_dir);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("ASR CSV cache miss for S100MISSING")
        );
        assert!(!temp_dir.exists());
        Ok(())
    }

    #[test]
    fn offline_mode_uses_cached_asr_csv() -> anyhow::Result<()> {
        let temp_dir = std::env::temp_dir().join(format!(
            "asrs_getter_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let csv_dir = temp_dir.join("S100CACHED").join("XBRL_TO_CSV");
        std::fs::create_dir_all(&csv_dir)?;
        let target = csv_dir.join("jpcrp030000-asr-001_E00000-000_2026-03-31_01_2026-04-01.csv");
        std::fs::write(&target, [])?;

        let found = find_cached_asr_csv("S100CACHED", &temp_dir)?;

        assert_eq!(found, target);
        std::fs::remove_dir_all(temp_dir)?;
        Ok(())
    }
}
