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
    fn accepts_edinet_document_id() {
        assert!(validate("S100XAN1").is_ok());
    }

    #[test]
    fn rejects_document_id_that_can_escape_a_directory() {
        assert!(validate("../file1").is_err());
        assert!(validate("S100/AN1").is_err());
    }

    #[test]
    fn rejects_document_id_with_an_unexpected_length_or_case() {
        assert!(validate("S100XAN").is_err());
        assert!(validate("s100xan1").is_err());
    }
}
