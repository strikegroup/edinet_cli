// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use std::io::Read;

const MAX_ENTRY_COUNT: usize = 20_000;
const MAX_UNCOMPRESSED_SIZE: u64 = 2 * 1024 * 1024 * 1024;

/// ZIP アーカイブを、展開先の外へ書き込まないよう検証しながら展開する。
pub async fn extract(
    archive_path: &std::path::Path,
    extract_to: &std::path::Path,
) -> anyhow::Result<()> {
    let archive_path = archive_path.to_owned();
    let extract_to = extract_to.to_owned();

    tokio::task::spawn_blocking(move || extract_blocking(&archive_path, &extract_to))
        .await
        .context("ZIP extraction task failed")??;
    Ok(())
}

fn extract_blocking(
    archive_path: &std::path::Path,
    extract_to: &std::path::Path,
) -> anyhow::Result<()> {
    let file = std::fs::File::open(archive_path).context("failed to open zip archive")?;
    let mut archive = zip::ZipArchive::new(file).context("failed to read zip archive")?;

    if archive.len() > MAX_ENTRY_COUNT {
        return Err(anyhow::anyhow!(
            "ZIP archive contains too many entries: {} (maximum {})",
            archive.len(),
            MAX_ENTRY_COUNT
        ));
    }

    create_safe_directory(extract_to)?;
    let mut extracted_size = 0_u64;

    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .context("failed to access zip entry")?;
        let relative_path = file
            .enclosed_name()
            .ok_or_else(|| anyhow::anyhow!("ZIP entry contains an unsafe path: {}", file.name()))?;
        if relative_path.as_os_str().is_empty() {
            return Err(anyhow::anyhow!("ZIP entry path must not be empty"));
        }

        let declared_total = extracted_size
            .checked_add(file.size())
            .ok_or_else(|| anyhow::anyhow!("ZIP archive uncompressed size overflow"))?;
        if declared_total > MAX_UNCOMPRESSED_SIZE {
            return Err(anyhow::anyhow!(
                "ZIP archive exceeds the maximum uncompressed size of {} bytes",
                MAX_UNCOMPRESSED_SIZE
            ));
        }

        let out_path = extract_to.join(&relative_path);
        ensure_no_symlink_components(extract_to, &relative_path)?;

        if file.is_dir() {
            create_safe_directory(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            create_safe_directory(parent)?;
        }
        reject_existing_symlink(&out_path)?;

        let mut out_file = std::fs::File::create(&out_path)
            .with_context(|| format!("failed to create extracted file {}", out_path.display()))?;
        let remaining = MAX_UNCOMPRESSED_SIZE - extracted_size;
        let copied = std::io::copy(&mut file.take(remaining + 1), &mut out_file)
            .context("failed to write extracted file")?;
        if copied > remaining {
            drop(out_file);
            let _ = std::fs::remove_file(&out_path);
            return Err(anyhow::anyhow!(
                "ZIP archive exceeds the maximum uncompressed size of {} bytes",
                MAX_UNCOMPRESSED_SIZE
            ));
        }
        extracted_size += copied;
    }

    Ok(())
}

fn create_safe_directory(path: &std::path::Path) -> anyhow::Result<()> {
    if path.exists() {
        let metadata = std::fs::symlink_metadata(path)
            .with_context(|| format!("failed to inspect directory {}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(anyhow::anyhow!(
                "ZIP extraction directory is not a regular directory: {}",
                path.display()
            ));
        }
        return Ok(());
    }

    std::fs::create_dir_all(path)
        .with_context(|| format!("failed to create directory {}", path.display()))
}

fn ensure_no_symlink_components(
    root: &std::path::Path,
    relative_path: &std::path::Path,
) -> anyhow::Result<()> {
    let mut current = root.to_owned();
    for component in relative_path.components() {
        current.push(component);
        if !current.exists() {
            continue;
        }
        reject_existing_symlink(&current)?;
    }
    Ok(())
}

fn reject_existing_symlink(path: &std::path::Path) -> anyhow::Result<()> {
    if let Ok(metadata) = std::fs::symlink_metadata(path)
        && metadata.file_type().is_symlink()
    {
        return Err(anyhow::anyhow!(
            "ZIP extraction path contains a symbolic link: {}",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn unique_temp_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "edinet_zip_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time must be after UNIX epoch")
                .as_nanos()
        ))
    }

    fn write_zip(path: &std::path::Path, entry_name: &str, content: &[u8]) {
        let file = std::fs::File::create(path).expect("ZIP file must be creatable");
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file(entry_name, zip::write::SimpleFileOptions::default())
            .expect("ZIP entry must be creatable");
        writer
            .write_all(content)
            .expect("ZIP content must be writable");
        writer.finish().expect("ZIP must be finishable");
    }

    #[test]
    fn extracts_a_regular_entry() {
        let temp_dir = unique_temp_dir("regular");
        std::fs::create_dir_all(&temp_dir).expect("temporary directory must be creatable");
        let archive_path = temp_dir.join("archive.zip");
        let extract_to = temp_dir.join("output");
        write_zip(&archive_path, "nested/report.csv", b"content");

        extract_blocking(&archive_path, &extract_to).expect("regular ZIP must be extracted");

        assert_eq!(
            std::fs::read(extract_to.join("nested/report.csv"))
                .expect("extracted file must be readable"),
            b"content"
        );
        std::fs::remove_dir_all(temp_dir).expect("temporary directory must be removable");
    }

    #[test]
    fn rejects_an_entry_that_escapes_the_destination() {
        let temp_dir = unique_temp_dir("traversal");
        std::fs::create_dir_all(&temp_dir).expect("temporary directory must be creatable");
        let archive_path = temp_dir.join("archive.zip");
        let extract_to = temp_dir.join("output");
        write_zip(&archive_path, "../escaped.txt", b"unsafe");

        let error = extract_blocking(&archive_path, &extract_to)
            .expect_err("unsafe ZIP entry must be rejected");

        assert!(error.to_string().contains("unsafe path"));
        assert!(!temp_dir.join("escaped.txt").exists());
        std::fs::remove_dir_all(temp_dir).expect("temporary directory must be removable");
    }
}
