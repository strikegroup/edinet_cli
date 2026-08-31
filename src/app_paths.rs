// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

const QUALIFIER: &str = "jp.co";
const ORGANIZATION: &str = "strike";
const APPLICATION: &str = "edinet_cli";

fn project_dirs() -> anyhow::Result<directories::ProjectDirs> {
    directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).ok_or_else(|| {
        anyhow::anyhow!("failed to resolve application data directory from the operating system")
    })
}

/// アプリのデータ保存先ディレクトリを返す。
///
/// DB ファイルや CSV キャッシュの既定ルートとして利用する。
pub fn default_data_local_dir() -> anyhow::Result<std::path::PathBuf> {
    Ok(project_dirs()?.data_local_dir().to_path_buf())
}

/// アプリの設定ディレクトリを返す。
///
/// 例: macOS では `~/Library/Application Support/jp.co.strike.edinet_cli/config`
pub fn config_dir() -> anyhow::Result<std::path::PathBuf> {
    Ok(project_dirs()?.config_dir().to_path_buf())
}

/// 設定ファイル `config.toml` のパスを返す。
pub fn config_file_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

/// 既定の SQLite DB ファイルパスを返す。
pub fn default_database_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(default_data_local_dir()?.join("asrs.db"))
}

/// 既定の CSV キャッシュディレクトリを返す。
pub fn default_csv_cache_dir() -> anyhow::Result<std::path::PathBuf> {
    Ok(default_data_local_dir()?.join("csv"))
}

/// 現在利用する SQLite DB ファイルパスを返す。
///
/// `DATABASE_URL` が設定されていればそこから解決し、未設定の場合は既定パスを返す。
/// `sqlite::memory:` のようなファイル実体を持たない URL はサポートしない。
pub fn current_database_path() -> anyhow::Result<std::path::PathBuf> {
    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        return parse_sqlite_file_path_from_url(&database_url);
    }

    default_database_path()
}

/// 現在利用する CSV キャッシュディレクトリを返す。
pub fn current_csv_cache_dir() -> anyhow::Result<std::path::PathBuf> {
    if let Ok(path) = std::env::var("ASRS_CSV_CACHE_DIR") {
        return Ok(std::path::PathBuf::from(path));
    }

    default_csv_cache_dir()
}

fn parse_sqlite_file_path_from_url(database_url: &str) -> anyhow::Result<std::path::PathBuf> {
    if database_url == "sqlite::memory:" {
        return Err(anyhow::anyhow!(
            "DATABASE_URL=sqlite::memory: is not supported by clear"
        ));
    }

    let path = database_url
        .strip_prefix("sqlite://")
        .ok_or_else(|| anyhow::anyhow!("unsupported DATABASE_URL for clear: {}", database_url))?;
    let path = path.split('?').next().unwrap_or(path);
    if path.is_empty() {
        return Err(anyhow::anyhow!(
            "DATABASE_URL does not contain a SQLite file path: {}",
            database_url
        ));
    }

    Ok(std::path::PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::parse_sqlite_file_path_from_url;
    use std::path::PathBuf;

    #[test]
    fn parses_absolute_sqlite_database_url() -> anyhow::Result<()> {
        let path = parse_sqlite_file_path_from_url("sqlite:///tmp/asr.db")?;
        assert_eq!(path, PathBuf::from("/tmp/asr.db"));
        Ok(())
    }

    #[test]
    fn strips_query_parameters_from_sqlite_database_url() -> anyhow::Result<()> {
        let path = parse_sqlite_file_path_from_url("sqlite:///tmp/asr.db?mode=rwc")?;
        assert_eq!(path, PathBuf::from("/tmp/asr.db"));
        Ok(())
    }
}
