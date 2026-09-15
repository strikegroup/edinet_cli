// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

#[derive(clap::Args, Debug)]
pub struct ClearArgs {
    #[arg(long, short = 'y', help = "確認を省略してローカルデータを削除します")]
    yes: bool,
}

struct ClearTarget {
    label: &'static str,
    path: PathBuf,
    size: u64,
    is_dir: bool,
}

/// 保存済みのローカルデータを、確認後に削除する。
pub fn clear_local_data(args: ClearArgs) -> anyhow::Result<()> {
    let database_path = crate::app_paths::current_database_path()?;
    let csv_cache_dir = crate::app_paths::current_csv_cache_dir()?;
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    clear_paths(
        &database_path,
        &csv_cache_dir,
        args.yes,
        &mut stdin.lock(),
        &mut stdout.lock(),
    )
}

fn clear_paths(
    database_path: &Path,
    csv_cache_dir: &Path,
    yes: bool,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> anyhow::Result<()> {
    let targets = collect_targets(database_path, csv_cache_dir)?;

    if targets.is_empty() {
        writeln!(output, "No local data to delete.")?;
        return Ok(());
    }

    writeln!(output, "The following local data will be deleted:")?;
    for target in &targets {
        writeln!(
            output,
            "  {}: {} ({})",
            target.label,
            target.path.display(),
            format_size(target.size)
        )?;
    }
    let total_size = targets.iter().map(|target| target.size).sum();
    writeln!(output, "Total: {}", format_size(total_size))?;

    if !yes {
        write!(output, "Delete? [y/N]: ")?;
        output.flush()?;

        let mut answer = String::new();
        input.read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            writeln!(output, "Cancelled.")?;
            return Ok(());
        }
    }

    for target in &targets {
        if target.is_dir {
            remove_dir_if_exists(&target.path, target.label)?;
        } else {
            remove_if_exists(&target.path, target.label)?;
        }
    }

    writeln!(output, "Local data deleted.")?;
    Ok(())
}

fn collect_targets(database_path: &Path, csv_cache_dir: &Path) -> anyhow::Result<Vec<ClearTarget>> {
    let candidates = [
        ("SQLite DB", database_path.to_path_buf(), false),
        (
            "SQLite SHM",
            sqlite_sidecar_path(database_path, "-shm"),
            false,
        ),
        (
            "SQLite WAL",
            sqlite_sidecar_path(database_path, "-wal"),
            false,
        ),
        ("CSV Cache", csv_cache_dir.to_path_buf(), true),
    ];

    candidates
        .into_iter()
        .filter(|(_, path, _)| path.exists())
        .map(|(label, path, is_dir)| {
            let size = path_size(&path)
                .with_context(|| format!("failed to calculate size of {}", path.display()))?;
            Ok(ClearTarget {
                label,
                path,
                size,
                is_dir,
            })
        })
        .collect()
}

fn path_size(path: &Path) -> anyhow::Result<u64> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir() {
        return Ok(metadata.len());
    }

    std::fs::read_dir(path)?.try_fold(0_u64, |total, entry| {
        let entry = entry?;
        let size = path_size(&entry.path())?;
        Ok(total.saturating_add(size))
    })
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn sqlite_sidecar_path(database_path: &Path, suffix: &str) -> PathBuf {
    let mut sidecar = std::ffi::OsString::from(database_path.as_os_str());
    sidecar.push(suffix);
    std::path::PathBuf::from(sidecar)
}

fn remove_if_exists(path: &Path, label: &str) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    std::fs::remove_file(path)
        .with_context(|| format!("failed to remove {} {}", label, path.display()))
}

fn remove_dir_if_exists(path: &Path, label: &str) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    std::fs::remove_dir_all(path)
        .with_context(|| format!("failed to remove {} {}", label, path.display()))
}

#[cfg(test)]
mod tests {
    use super::clear_paths;
    use std::io::Cursor;

    fn unique_temp_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "edinet-clear-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock must be after epoch")
                .as_nanos()
        ))
    }

    fn create_local_data(name: &str) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
        let root = unique_temp_dir(name);
        let database = root.join("asrs.db");
        let cache = root.join("csv");
        std::fs::create_dir_all(cache.join("document"))?;
        std::fs::write(&database, vec![0; 1024])?;
        std::fs::write(cache.join("document").join("data.csv"), vec![0; 512])?;
        Ok((database, cache))
    }

    #[test]
    fn clearは削除対象と合計容量を示しyの確認後に削除する() -> anyhow::Result<()> {
        let (database, cache) = create_local_data("confirmed")?;
        let mut input = Cursor::new(b"y\n");
        let mut output = Vec::new();

        clear_paths(&database, &cache, false, &mut input, &mut output)?;

        assert!(!database.exists());
        assert!(!cache.exists());
        let output = String::from_utf8(output)?;
        assert!(output.contains("SQLite DB:"));
        assert!(output.contains("CSV Cache:"));
        assert!(output.contains("Total: 1.5 KiB"));
        assert!(output.contains("Delete? [y/N]:"));
        Ok(())
    }

    #[test]
    fn clearはy以外の回答では何も削除しない() -> anyhow::Result<()> {
        let (database, cache) = create_local_data("cancelled")?;
        let root = database
            .parent()
            .expect("database must have a parent")
            .to_path_buf();
        let mut input = Cursor::new(b"n\n");
        let mut output = Vec::new();

        clear_paths(&database, &cache, false, &mut input, &mut output)?;

        assert!(database.exists());
        assert!(cache.exists());
        assert!(String::from_utf8(output)?.contains("Cancelled."));
        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn clearのyesオプションは確認だけを省略して削除する() -> anyhow::Result<()> {
        let (database, cache) = create_local_data("forced")?;
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Vec::new();

        clear_paths(&database, &cache, true, &mut input, &mut output)?;

        assert!(!database.exists());
        assert!(!cache.exists());
        assert!(!String::from_utf8(output)?.contains("[y/N]"));
        Ok(())
    }
}
