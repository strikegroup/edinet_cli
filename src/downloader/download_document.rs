// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use std::io::Write;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum DocType {
    Xbrl,
    Pdf,
    Csv,
}

/// 書類種別に応じて書類取得 API からバイナリデータをダウンロードする。
pub async fn download_document(
    doc_type: &DocType,
    doc_id: &str,
    api_key: &str,
    dest: &std::path::Path,
    extract: bool,
) -> anyhow::Result<std::path::PathBuf> {
    crate::document_id::validate(doc_id)?;
    match doc_type {
        DocType::Xbrl => download_xbrl(doc_id, api_key, dest, extract).await,
        DocType::Pdf => download_pdf(doc_id, api_key, dest, extract).await,
        DocType::Csv => download_csv(doc_id, api_key, dest, extract).await,
    }
}

/// 提出本文書、監査報告書、XBRL 一式の ZIP をダウンロードする。
async fn download_xbrl(
    doc_id: &str,
    api_key: &str,
    dest: &std::path::Path,
    extract: bool,
) -> anyhow::Result<std::path::PathBuf> {
    let file_name = format!("{doc_id}-xbrl.zip");
    let output_path = if dest.is_dir() {
        dest.join(&file_name)
    } else if dest.is_file() || dest.extension().is_some() {
        if !dest
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            return Err(anyhow::anyhow!("destination file extension must be .zip"));
        }
        dest.to_path_buf()
    } else {
        dest.join(&file_name)
    };

    let url = format!(
        "https://api.edinet-fsa.go.jp/api/v2/documents/{}?type=1&Subscription-Key={}",
        doc_id, api_key
    );
    let bytes = reqwest::get(&url)
        .await
        .with_context(|| format!("failed to fetch document {}", doc_id))?
        .error_for_status()
        .context("error response")?
        .bytes()
        .await
        .context("failed to read bytes")?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).context("failed to create parent directory")?;
    }
    let mut out_file = std::fs::File::create(&output_path).context("failed to create file")?;
    out_file.write_all(&bytes).context("failed to write file")?;

    if extract {
        let extract_to = output_path.with_extension("");
        crate::zip_archive::extract(&output_path, &extract_to).await?;
    }

    Ok(output_path)
}

/// PDF をダウンロードする。
async fn download_pdf(
    doc_id: &str,
    api_key: &str,
    dest: &std::path::Path,
    extract: bool,
) -> anyhow::Result<std::path::PathBuf> {
    if extract {
        return Err(anyhow::anyhow!(
            "--extract can only be used with xbrl or csv"
        ));
    }

    let file_name = format!("{doc_id}.pdf");
    let output_path = if dest.is_dir() {
        dest.join(&file_name)
    } else if dest.is_file() || dest.extension().is_some() {
        if !dest
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
        {
            return Err(anyhow::anyhow!("destination file extension must be .pdf"));
        }
        dest.to_path_buf()
    } else {
        dest.join(&file_name)
    };

    let url = format!(
        "https://api.edinet-fsa.go.jp/api/v2/documents/{}?type=2&Subscription-Key={}",
        doc_id, api_key
    );
    let bytes = reqwest::get(&url)
        .await
        .with_context(|| format!("failed to fetch document {}", doc_id))?
        .error_for_status()
        .context("error response")?
        .bytes()
        .await
        .context("failed to read bytes")?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).context("failed to create parent directory")?;
    }
    let mut out_file = std::fs::File::create(&output_path).context("failed to create file")?;
    out_file.write_all(&bytes).context("failed to write file")?;

    Ok(output_path)
}

/// XBRL 変換 CSV の ZIP をダウンロードする。
async fn download_csv(
    doc_id: &str,
    api_key: &str,
    dest: &std::path::Path,
    extract: bool,
) -> anyhow::Result<std::path::PathBuf> {
    let file_name = format!("{doc_id}-csv.zip");
    let output_path = if dest.is_dir() {
        dest.join(&file_name)
    } else if dest.is_file() || dest.extension().is_some() {
        if !dest
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            return Err(anyhow::anyhow!("destination file extension must be .zip"));
        }
        dest.to_path_buf()
    } else {
        dest.join(&file_name)
    };

    let url = format!(
        "https://api.edinet-fsa.go.jp/api/v2/documents/{}?type=5&Subscription-Key={}",
        doc_id, api_key
    );
    let bytes = reqwest::get(&url)
        .await
        .with_context(|| format!("failed to fetch document {}", doc_id))?
        .error_for_status()
        .context("error response")?
        .bytes()
        .await
        .context("failed to read bytes")?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).context("failed to create parent directory")?;
    }
    let mut out_file = std::fs::File::create(&output_path).context("failed to create file")?;
    out_file.write_all(&bytes).context("failed to write file")?;

    if extract {
        let extract_to = output_path.with_extension("");
        crate::zip_archive::extract(&output_path, &extract_to).await?;
    }

    Ok(output_path)
}
