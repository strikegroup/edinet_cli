// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

//! EDINET の日次書類一覧を取得し、検索用 DB に反映するモジュール群。

/// `update` コマンドの引数と実行処理。
pub mod command;
/// EDINET 書類一覧 API レスポンスの型。
pub mod document_metadata;
/// EDINET API から日次書類一覧を取得する処理。
pub mod fetch_document_metadatas;
/// 取得した日次書類メタデータを DB へ保存する処理。
pub mod save_document_metadatas;
/// `update` / `status` コマンドの実行ロジック。
pub mod update_documents;
