// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use sea_orm::sea_query::Index;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbBackend,
    Schema,
};

use crate::store::entities::{document_metadata, updated_document_metadata};

/// SQLite 接続を開き、現在の Entity 定義から必要なテーブルを作成する。
///
/// `DATABASE_URL` が設定されていればそれを使い、未設定の場合は
/// アプリ既定の DB パス（`app_paths::default_database_path`）を使う。
pub async fn open_db_connection() -> anyhow::Result<DatabaseConnection> {
    let database_url = if let Ok(sqlite_url) = std::env::var("DATABASE_URL") {
        sqlite_database_url_with_create_mode(sqlite_url)
    } else {
        let db_path = crate::app_paths::default_database_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create database directory {}", parent.display())
            })?;
        }
        format!("sqlite://{}?mode=rwc", db_path.display())
    };

    let mut options = ConnectOptions::new(database_url);
    options.max_connections(5);

    let db = Database::connect(options)
        .await
        .context("failed to connect to SQLite database")?;
    create_schema(&db).await?;

    Ok(db)
}

/// 既存 SQLite DB だけに接続する。
///
/// 読み取り専用の状態確認で、DB ファイルや schema を作成しない接続として使う。
pub async fn open_existing_db_connection() -> anyhow::Result<Option<DatabaseConnection>> {
    let db_path = crate::app_paths::current_database_path()?;
    if !db_path.exists() {
        return Ok(None);
    }

    let database_url = format!("sqlite://{}?mode=rw", db_path.display());
    let mut options = ConnectOptions::new(database_url);
    options.max_connections(5);

    let db = Database::connect(options)
        .await
        .with_context(|| format!("failed to connect to SQLite database {}", db_path.display()))?;

    Ok(Some(db))
}

fn sqlite_database_url_with_create_mode(database_url: String) -> String {
    if !database_url.starts_with("sqlite:") || database_url.contains("mode=") {
        return database_url;
    }

    if database_url.contains('?') {
        format!("{database_url}&mode=rwc")
    } else {
        format!("{database_url}?mode=rwc")
    }
}

async fn create_schema(db: &DatabaseConnection) -> anyhow::Result<()> {
    let schema = Schema::new(DbBackend::Sqlite);
    let backend = DatabaseBackend::Sqlite;

    db.execute(
        backend.build(
            schema
                .create_table_from_entity(updated_document_metadata::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("failed to create updated_document_metadatas_list table")?;

    db.execute(
        backend.build(
            schema
                .create_table_from_entity(document_metadata::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("failed to create document_metadatas table")?;

    create_index(
        db,
        Index::create()
            .name("idx_document_metadatas_file_date")
            .table(document_metadata::Entity)
            .col(document_metadata::Column::FileDate)
            .if_not_exists(),
    )
    .await?;
    create_index(
        db,
        Index::create()
            .name("idx_document_metadatas_edinet_code")
            .table(document_metadata::Entity)
            .col(document_metadata::Column::EdinetCode)
            .if_not_exists(),
    )
    .await?;
    create_index(
        db,
        Index::create()
            .name("idx_document_metadatas_doc_id")
            .table(document_metadata::Entity)
            .col(document_metadata::Column::DocId)
            .if_not_exists(),
    )
    .await?;

    Ok(())
}

async fn create_index(
    db: &DatabaseConnection,
    statement: &mut sea_orm::sea_query::IndexCreateStatement,
) -> anyhow::Result<()> {
    db.execute(DatabaseBackend::Sqlite.build(statement))
        .await
        .context("failed to create database index")?;
    Ok(())
}
