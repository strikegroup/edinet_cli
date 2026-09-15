// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

const DOCUMENT_ID_LENGTH: usize = 8;

/// EDINET の書類管理番号として安全に扱える形式か検証する。
pub fn validate(value: &str) -> anyhow::Result<()> {
    if value.len() != DOCUMENT_ID_LENGTH
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    {
        return Err(anyhow::anyhow!(
            "document ID must be exactly 8 uppercase ASCII letters or digits"
        ));
    }

    Ok(())
}

/// clap の value parser として書類管理番号を検証する。
pub fn parse(value: &str) -> Result<String, String> {
    validate(value).map_err(|error| error.to_string())?;
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 書類idは8文字の半角英大文字と数字で受け付ける() {
        assert!(validate("S100XAN1").is_ok());
    }

    #[test]
    fn 書類idはパスとして解釈できる値や仕様外の形式を拒否する() {
        for value in ["../file1", "S100/AN1", "S100XAN", "s100xan1"] {
            assert!(validate(value).is_err(), "`{value}` は拒否されるべき");
        }
    }
}
