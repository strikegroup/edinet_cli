use anyhow::Context;

/// 指定 `doc_id` の XBRL CSV アーカイブをダウンロードし、展開先へ解凍する。
pub(in crate::getter) async fn download_and_extract_xbrl_csv_archive(
    doc_id: &str,
    api_key: &str,
    cache_root: &std::path::Path,
    extract_dir: &std::path::Path,
) -> anyhow::Result<()> {
    let archive_path = cache_root.join(format!("{doc_id}.zip"));
    download_xbrl_csv_archive(doc_id, &api_key, &archive_path).await?;
    extract_zip(&archive_path, &extract_dir).await?;
    Ok(())
}

async fn download_xbrl_csv_archive(
    doc_id: &str,
    api_key: &str,
    dest: &std::path::Path,
) -> anyhow::Result<()> {
    if dest.is_dir() {
        return Err(anyhow::anyhow!("destination path is a directory"));
    }

    let url = format!(
        "https://api.edinet-fsa.go.jp/api/v2/documents/{}?type=5&Subscription-Key={}",
        doc_id, api_key
    );
    let response = reqwest::get(&url)
        .await
        .with_context(|| format!("failed to fetch XBRL CSV archive for {}", doc_id))?;

    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let body = response
        .bytes()
        .await
        .with_context(|| format!("failed to read response body for {}", doc_id))?;

    if !status.is_success() || !content_type.starts_with("application/octet-stream") {
        let message = String::from_utf8_lossy(&body);
        return Err(anyhow::anyhow!(
            "failed to fetch XBRL CSV archive for {}: HTTP {} content-type {} body {}",
            doc_id,
            status,
            content_type,
            message
        ));
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).context("failed to create parent directory")?;
    }

    std::fs::write(dest, &body).context("failed to write XBRL CSV archive")?;

    Ok(())
}

async fn extract_zip(
    archive_path: &std::path::Path,
    extract_to: &std::path::Path,
) -> anyhow::Result<()> {
    let archive_path = archive_path.to_owned();
    let extract_to = extract_to.to_owned();

    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(archive_path).context("failed to open zip archive")?;
        let mut archive = zip::ZipArchive::new(file).context("failed to read zip archive")?;

        for index in 0..archive.len() {
            let mut file = archive
                .by_index(index)
                .context("failed to access zip entry")?;
            let out_path = extract_to.join(file.name());

            if file.is_dir() {
                std::fs::create_dir_all(&out_path).context("failed to create directory")?;
                continue;
            }

            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).context("failed to create parent directory")?;
            }

            let mut out_file =
                std::fs::File::create(&out_path).context("failed to create extracted file")?;
            std::io::copy(&mut file, &mut out_file).context("failed to write extracted file")?;
        }

        Ok(())
    })
    .await?
}
