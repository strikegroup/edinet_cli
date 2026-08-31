// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use byteorder::ReadBytesExt;

/// UTF-16LE/TSV 形式のローカル CSV を読み込み、CSV Reader を返す。
pub(in crate::getter) fn load_csv_from_path(
    file_path: &std::path::Path,
) -> anyhow::Result<csv::Reader<std::io::Cursor<Vec<u8>>>> {
    let file_content = std::fs::read(file_path).context("failed to read CSV file")?;
    let decoded_content = decode_utf16le(&file_content).context("failed to decode UTF-16LE")?;
    let cursor = std::io::Cursor::new(decoded_content.into_bytes());

    Ok(csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(cursor))
}

fn decode_utf16le(bytes: &[u8]) -> anyhow::Result<String> {
    use std::io::Cursor;

    let mut cursor = Cursor::new(bytes);
    let mut code_units = Vec::with_capacity(bytes.len() / 2);

    while let Ok(code_unit) = cursor.read_u16::<byteorder::LittleEndian>() {
        code_units.push(code_unit);
    }

    String::from_utf16(&code_units).context("failed to decode UTF-16LE")
}
