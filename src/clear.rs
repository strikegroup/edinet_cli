// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;

/// 保存済みのローカルデータを削除する。
pub fn clear_local_data() -> anyhow::Result<()> {
    let database_path = crate::app_paths::current_database_path()?;
    let csv_cache_dir = crate::app_paths::current_csv_cache_dir()?;

    remove_if_exists(&database_path, "database file")?;
    remove_if_exists(
        &sqlite_sidecar_path(&database_path, "-shm"),
        "database shared-memory file",
    )?;
    remove_if_exists(
        &sqlite_sidecar_path(&database_path, "-wal"),
        "database write-ahead log",
    )?;
    remove_dir_if_exists(&csv_cache_dir, "CSV cache directory")?;

    println!("Cleared local data");
    println!("database: {}", database_path.display());
    println!("csv cache: {}", csv_cache_dir.display());

    Ok(())
}

fn sqlite_sidecar_path(database_path: &std::path::Path, suffix: &str) -> std::path::PathBuf {
    let mut sidecar = std::ffi::OsString::from(database_path.as_os_str());
    sidecar.push(suffix);
    std::path::PathBuf::from(sidecar)
}

fn remove_if_exists(path: &std::path::Path, label: &str) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    std::fs::remove_file(path)
        .with_context(|| format!("failed to remove {} {}", label, path.display()))
}

fn remove_dir_if_exists(path: &std::path::Path, label: &str) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    std::fs::remove_dir_all(path)
        .with_context(|| format!("failed to remove {} {}", label, path.display()))
}
