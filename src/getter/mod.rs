// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

//! 有価証券報告書本文の取得・展開・CSV 解析を担当するモジュール群。

/// 有価証券報告書の抽出結果と出力型の定義。
pub mod asr_report;
/// `get` コマンドの引数と実行処理。
pub mod command;
/// 保存済み書類メタデータから ASR 対象を検索する処理。
pub mod find_asr_document;
/// doc_id を起点に有価証券報告書の構造化データを読み込む処理。
pub mod load_asr_report;
/// `get` コマンド向けの出力 DTO。
pub mod output;

mod download_document;
mod extract_asr_report;
mod extractors;
mod load_local_csv;
mod xbrl_fact;
