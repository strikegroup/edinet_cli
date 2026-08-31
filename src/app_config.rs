use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(clap::Args, Debug)]
pub struct SetupArgs {
    #[arg(long, short, value_name = "API_KEY", help = "登録する EDINET API キー")]
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    edinet_api_key: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ApiKeyRegistrationStatus {
    Registered,
    Missing,
    Invalid,
}

impl ApiKeyRegistrationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Registered => "registered",
            Self::Missing => "missing",
            Self::Invalid => "invalid",
        }
    }
}

/// EDINET API キーを `config.toml` に保存する。
///
/// 既存ファイルがある場合は上書きする。
pub fn save_api_key(api_key: &str) -> anyhow::Result<std::path::PathBuf> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow::anyhow!("api key must not be empty"));
    }

    let config_dir = crate::app_paths::config_dir()?;
    std::fs::create_dir_all(&config_dir).with_context(|| {
        format!(
            "failed to create configuration directory {}",
            config_dir.display()
        )
    })?;

    let config_path = crate::app_paths::config_file_path()?;
    let content = toml::to_string(&ConfigFile {
        edinet_api_key: api_key.to_owned(),
    })
    .context("failed to serialize config file")?;
    std::fs::write(&config_path, content)
        .with_context(|| format!("failed to write config file {}", config_path.display()))?;

    Ok(config_path)
}

/// `config.toml` から EDINET API キーを読み込む。
///
/// 設定ファイルが存在しない場合や値が空の場合はエラーを返す。
pub fn load_api_key() -> anyhow::Result<String> {
    let config_path = crate::app_paths::config_file_path()?;
    let content = std::fs::read_to_string(&config_path).with_context(|| {
        format!(
            "failed to read config file {}. run `cargo run -- setup --key <EDINET_API_KEY>` first or pass `--key`",
            config_path.display()
        )
    })?;
    let config: ConfigFile = toml::from_str(&content)
        .with_context(|| format!("failed to parse config file {}", config_path.display()))?;
    let api_key = config.edinet_api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow::anyhow!(
            "edinet_api_key is empty in {}",
            config_path.display()
        ));
    }

    Ok(api_key.to_owned())
}

/// 保存済み EDINET API キーの登録状態を返す。
pub fn api_key_registration_status() -> anyhow::Result<ApiKeyRegistrationStatus> {
    let config_path = crate::app_paths::config_file_path()?;
    let content = match std::fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ApiKeyRegistrationStatus::Missing);
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read config file {}", config_path.display()));
        }
    };

    let Ok(config) = toml::from_str::<ConfigFile>(&content) else {
        return Ok(ApiKeyRegistrationStatus::Invalid);
    };
    if config.edinet_api_key.trim().is_empty() {
        return Ok(ApiKeyRegistrationStatus::Invalid);
    }

    Ok(ApiKeyRegistrationStatus::Registered)
}

/// 実行時 API キーを解決する。
///
/// - `cli_api_key` が指定されていればそれを優先
/// - 未指定なら `config.toml` の保存済みキーを利用
pub fn resolve_api_key(cli_api_key: Option<&str>) -> anyhow::Result<String> {
    if let Some(api_key) = cli_api_key {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            return Err(anyhow::anyhow!("api key must not be empty"));
        }
        return Ok(api_key.to_owned());
    }

    load_api_key()
}

pub fn run_setup(args: SetupArgs) -> anyhow::Result<()> {
    let path = save_api_key(&args.key)?;
    println!("Saved EDINET API key to {}", path.display());
    Ok(())
}
